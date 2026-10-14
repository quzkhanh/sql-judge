use std::time::Duration;

use futures_util::StreamExt;
use judge_core::{Column, LogicalType, QueryLimits, QueryResult, SqlValue};
use thiserror::Error;
use tiberius::ToSql;
use tiberius::{AuthMethod, Client, ColumnData, ColumnType, Config, EncryptionLevel};
use tokio::net::TcpStream;
use tokio::runtime::{Builder, Runtime};
use tokio_util::compat::{Compat, TokioAsyncWriteCompatExt};

type TdsClient = Client<Compat<TcpStream>>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SqlServerTlsMode {
    VerifyFull,
    VerifyCa,
    Require,
    Disable,
    TrustServerCertificate,
}

#[derive(Clone, PartialEq, Eq)]
pub struct SqlServerConfig {
    pub host: String,
    pub port: u16,
    pub database: String,
    pub username: String,
    pub password: String,
    pub tls_mode: SqlServerTlsMode,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum SqlServerError {
    #[error("SQL Server connection profile is invalid: {0}")]
    InvalidProfile(&'static str),
    #[error("SQL Server TLS policy rejected")]
    TlsPolicyRejected,
    #[error("SQL Server TLS negotiation failed")]
    TlsFailed,
    #[error("SQL Server authentication failed")]
    AuthenticationFailed,
    #[error("SQL Server is unreachable")]
    ServerUnreachable,
    #[error("SQL Server query limits are invalid: {0}")]
    InvalidLimits(String),
    #[error("T-SQL query was rejected by the read-only policy")]
    QueryRejected,
    #[error("SQL Server query timed out")]
    QueryTimeout,
    #[error("SQL Server result exceeded its limit")]
    ResultLimitExceeded,
    #[error("SQL Server operation failed")]
    DatabaseError,
    #[error("SQL Server returned an unsupported value type: {0}")]
    UnsupportedType(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SqlServerConnectionInfo {
    pub database: String,
    pub username: String,
    pub server_version: String,
    pub read_only: bool,
}

#[derive(Debug, Clone, Copy)]
struct PermissionSnapshot {
    server_escalation: Option<bool>,
    database_escalation: Option<bool>,
    database_write: Option<bool>,
    object_write: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SqlServerColumn {
    pub name: String,
    pub data_type: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SqlServerTable {
    pub schema: String,
    pub name: String,
    pub kind: String,
    pub columns: Vec<SqlServerColumn>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SqlServerMetadata {
    pub tables: Vec<SqlServerTable>,
    pub truncated: bool,
}

pub struct SqlServerExplorer {
    config: SqlServerConfig,
}

impl SqlServerExplorer {
    pub fn connect(config: SqlServerConfig) -> Result<Self, SqlServerError> {
        validate_config(&config)?;
        let explorer = Self { config };
        let (_runtime, _client) = explorer.open_client(Duration::from_secs(5))?;
        Ok(explorer)
    }

    pub fn test_connection(&self) -> Result<SqlServerConnectionInfo, SqlServerError> {
        let (runtime, mut client) = self.open_client(Duration::from_secs(2))?;
        runtime.block_on(async {
            let stream = client
                .query(
                    "SELECT DB_NAME(), SUSER_SNAME(),
                            CONVERT(nvarchar(128), SERVERPROPERTY('ProductVersion')),
                            CONVERT(bit, CASE WHEN ISNULL(IS_SRVROLEMEMBER('sysadmin'), 1) = 1
                                OR ISNULL(IS_SRVROLEMEMBER('serveradmin'), 1) = 1
                                OR ISNULL(IS_SRVROLEMEMBER('securityadmin'), 1) = 1
                                OR ISNULL(IS_SRVROLEMEMBER('setupadmin'), 1) = 1
                                OR ISNULL(IS_SRVROLEMEMBER('processadmin'), 1) = 1
                                OR ISNULL(IS_SRVROLEMEMBER('diskadmin'), 1) = 1
                                OR ISNULL(IS_SRVROLEMEMBER('dbcreator'), 1) = 1
                                OR ISNULL(IS_SRVROLEMEMBER('bulkadmin'), 1) = 1
                                OR ISNULL(HAS_PERMS_BY_NAME(NULL, 'SERVER', 'CONTROL SERVER'), 1) = 1
                                OR ISNULL(HAS_PERMS_BY_NAME(NULL, 'SERVER', 'ALTER ANY DATABASE'), 1) = 1
                                OR ISNULL(HAS_PERMS_BY_NAME(NULL, 'SERVER', 'ALTER ANY LOGIN'), 1) = 1
                                OR ISNULL(HAS_PERMS_BY_NAME(NULL, 'SERVER', 'IMPERSONATE ANY LOGIN'), 1) = 1
                                THEN 1 ELSE 0 END),
                            CONVERT(bit, CASE WHEN ISNULL(IS_MEMBER('db_owner'), 1) = 1
                                OR ISNULL(IS_MEMBER('db_securityadmin'), 1) = 1
                                OR ISNULL(IS_MEMBER('db_accessadmin'), 1) = 1
                                OR ISNULL(IS_MEMBER('db_ddladmin'), 1) = 1
                                OR ISNULL(IS_MEMBER('db_datawriter'), 1) = 1
                                THEN 1 ELSE 0 END),
                            CONVERT(bit, CASE WHEN
                                ISNULL(HAS_PERMS_BY_NAME(DB_NAME(), 'DATABASE', 'CONTROL'), 1) = 1
                                OR ISNULL(HAS_PERMS_BY_NAME(DB_NAME(), 'DATABASE', 'ALTER'), 1) = 1
                                OR ISNULL(HAS_PERMS_BY_NAME(DB_NAME(), 'DATABASE', 'TAKE OWNERSHIP'), 1) = 1
                                OR ISNULL(HAS_PERMS_BY_NAME(DB_NAME(), 'DATABASE', 'IMPERSONATE'), 1) = 1
                                OR ISNULL(HAS_PERMS_BY_NAME(DB_NAME(), 'DATABASE', 'EXECUTE'), 1) = 1
                                OR ISNULL(HAS_PERMS_BY_NAME(DB_NAME(), 'DATABASE', 'INSERT'), 1) = 1
                                OR ISNULL(HAS_PERMS_BY_NAME(DB_NAME(), 'DATABASE', 'UPDATE'), 1) = 1
                                OR ISNULL(HAS_PERMS_BY_NAME(DB_NAME(), 'DATABASE', 'DELETE'), 1) = 1
                                OR ISNULL(HAS_PERMS_BY_NAME(DB_NAME(), 'DATABASE', 'CREATE TABLE'), 1) = 1
                                OR ISNULL(HAS_PERMS_BY_NAME(DB_NAME(), 'DATABASE', 'CREATE SCHEMA'), 1) = 1
                                OR ISNULL(HAS_PERMS_BY_NAME(DB_NAME(), 'DATABASE', 'CREATE PROCEDURE'), 1) = 1
                                OR ISNULL(HAS_PERMS_BY_NAME(DB_NAME(), 'DATABASE', 'CREATE VIEW'), 1) = 1
                                OR ISNULL(HAS_PERMS_BY_NAME(DB_NAME(), 'DATABASE', 'CREATE FUNCTION'), 1) = 1
                                OR ISNULL(HAS_PERMS_BY_NAME(DB_NAME(), 'DATABASE', 'CREATE TYPE'), 1) = 1
                                OR ISNULL(HAS_PERMS_BY_NAME(DB_NAME(), 'DATABASE', 'ALTER ANY USER'), 1) = 1
                                OR ISNULL(HAS_PERMS_BY_NAME(DB_NAME(), 'DATABASE', 'ALTER ANY ROLE'), 1) = 1
                                OR ISNULL(HAS_PERMS_BY_NAME(DB_NAME(), 'DATABASE', 'CREATE ASSEMBLY'), 1) = 1
                                OR ISNULL(HAS_PERMS_BY_NAME(DB_NAME(), 'DATABASE', 'REFERENCES'), 1) = 1
                                THEN 1 ELSE 0 END),
                            CONVERT(bit, CASE WHEN EXISTS (
                                SELECT 1
                                FROM sys.schemas AS s
                                WHERE ISNULL(HAS_PERMS_BY_NAME(s.name, 'SCHEMA', 'ALTER'), 1) = 1
                                   OR ISNULL(HAS_PERMS_BY_NAME(s.name, 'SCHEMA', 'CONTROL'), 1) = 1
                                   OR ISNULL(HAS_PERMS_BY_NAME(s.name, 'SCHEMA', 'TAKE OWNERSHIP'), 1) = 1
                            ) OR EXISTS (
                                SELECT 1
                                FROM sys.objects AS o
                                JOIN sys.schemas AS s ON s.schema_id = o.schema_id
                                WHERE o.is_ms_shipped = 0
                                  AND (
                                    ISNULL(HAS_PERMS_BY_NAME(QUOTENAME(s.name) + '.' + QUOTENAME(o.name), 'OBJECT', 'INSERT'), 1) = 1
                                    OR ISNULL(HAS_PERMS_BY_NAME(QUOTENAME(s.name) + '.' + QUOTENAME(o.name), 'OBJECT', 'UPDATE'), 1) = 1
                                    OR ISNULL(HAS_PERMS_BY_NAME(QUOTENAME(s.name) + '.' + QUOTENAME(o.name), 'OBJECT', 'DELETE'), 1) = 1
                                    OR ISNULL(HAS_PERMS_BY_NAME(QUOTENAME(s.name) + '.' + QUOTENAME(o.name), 'OBJECT', 'ALTER'), 1) = 1
                                    OR ISNULL(HAS_PERMS_BY_NAME(QUOTENAME(s.name) + '.' + QUOTENAME(o.name), 'OBJECT', 'CONTROL'), 1) = 1
                                    OR ISNULL(HAS_PERMS_BY_NAME(QUOTENAME(s.name) + '.' + QUOTENAME(o.name), 'OBJECT', 'EXECUTE'), 1) = 1
                                  )
                            ) THEN 1 ELSE 0 END)",
                    &[],
                )
                .await
                .map_err(|error| map_driver_error_for(error, DriverOperation::Query))?;
            let row = stream
                .into_row()
                .await
                .map_err(|error| map_driver_error_for(error, DriverOperation::Query))?
                .ok_or(SqlServerError::DatabaseError)?;
            Ok(SqlServerConnectionInfo {
                database: text_cell(&row, 0)?,
                username: text_cell(&row, 1)?,
                server_version: text_cell(&row, 2)?,
                read_only: read_only_from_permissions(PermissionSnapshot {
                    server_escalation: row
                        .try_get::<bool, _>(3)
                        .map_err(|_| SqlServerError::DatabaseError)?,
                    database_escalation: row
                        .try_get::<bool, _>(4)
                        .map_err(|_| SqlServerError::DatabaseError)?,
                    database_write: row
                        .try_get::<bool, _>(5)
                        .map_err(|_| SqlServerError::DatabaseError)?,
                    object_write: row
                        .try_get::<bool, _>(6)
                        .map_err(|_| SqlServerError::DatabaseError)?,
                }),
            })
        })
    }

    pub fn list_metadata(&self, limits: QueryLimits) -> Result<SqlServerMetadata, SqlServerError> {
        validate_limits(&limits)?;
        let (runtime, mut client) = self.open_client(timeout(&limits))?;
        runtime.block_on(async move {
            in_read_only_transaction(&mut client, &limits, move |client| {
                Box::pin(async move {
                    let requested = limits.max_rows.saturating_add(1) as i32;
                    let metadata_sql = format!(
                        "SELECT TOP ({requested}) s.name, o.name, o.type_desc, c.name, ty.name
                     FROM sys.objects AS o
                     JOIN sys.schemas AS s ON s.schema_id = o.schema_id
                     JOIN sys.columns AS c ON c.object_id = o.object_id
                     JOIN sys.types AS ty ON ty.user_type_id = c.user_type_id
                     WHERE o.type IN ('U', 'V') AND o.is_ms_shipped = 0
                     ORDER BY s.name, o.name, c.column_id"
                    );
                    let rows = query_rows(
                        client,
                        &metadata_sql,
                        &[],
                        limits.max_rows.saturating_add(1) as usize,
                        limits.max_bytes,
                    )
                    .await?;
                    let mut tables = Vec::new();
                    let mut bytes = 0_u32;
                    for row in rows.iter().take(limits.max_rows as usize) {
                        let schema = text_cell(row, 0)?;
                        let name = text_cell(row, 1)?;
                        let kind = text_cell(row, 2)?;
                        let column_name = text_cell(row, 3)?;
                        let data_type = text_cell(row, 4)?;
                        bytes = bytes
                            .checked_add(
                                u32::try_from(
                                    schema.len()
                                        + name.len()
                                        + kind.len()
                                        + column_name.len()
                                        + data_type.len(),
                                )
                                .map_err(|_| SqlServerError::ResultLimitExceeded)?,
                            )
                            .ok_or(SqlServerError::ResultLimitExceeded)?;
                        if bytes > limits.max_bytes {
                            return Err(SqlServerError::ResultLimitExceeded);
                        }
                        if let Some(table) =
                            tables.iter_mut().find(|table: &&mut SqlServerTable| {
                                table.schema == schema && table.name == name
                            })
                        {
                            table.columns.push(SqlServerColumn {
                                name: column_name,
                                data_type,
                            });
                        } else {
                            tables.push(SqlServerTable {
                                schema,
                                name,
                                kind,
                                columns: vec![SqlServerColumn {
                                    name: column_name,
                                    data_type,
                                }],
                            });
                        }
                    }
                    Ok(SqlServerMetadata {
                        truncated: rows.len() > limits.max_rows as usize,
                        tables,
                    })
                })
            })
            .await
        })
    }

    pub fn execute_select(
        &self,
        sql: &str,
        limits: QueryLimits,
    ) -> Result<QueryResult, SqlServerError> {
        validate_read_only_tsql(sql)?;
        validate_limits(&limits)?;
        let query_sql = sql.to_owned();
        let (runtime, mut client) = self.open_client(timeout(&limits))?;
        runtime.block_on(async move {
            in_read_only_transaction(&mut client, &limits, move |client| {
                Box::pin(async move {
                    let dependency_rows = query_rows(
                        client,
                        "SELECT DB_NAME(), source_database, source_schema, source_table,
                                error_number, error_type
                        FROM sys.dm_exec_describe_first_result_set(@P1, NULL, 1)
                        WHERE source_database IS NOT NULL OR source_schema IS NOT NULL
                           OR source_table IS NOT NULL OR error_number IS NOT NULL
                           OR error_type IS NOT NULL",
                        &[&query_sql],
                        limits.max_rows as usize,
                        limits.max_bytes,
                    )
                    .await?;
                    for row in dependency_rows {
                        let current_database = text_cell(&row, 0)?;
                        let referenced_database = row
                            .try_get::<&str, _>(1)
                            .map_err(|_| SqlServerError::DatabaseError)?;
                        let referenced_schema = row
                            .try_get::<&str, _>(2)
                            .map_err(|_| SqlServerError::DatabaseError)?;
                        let referenced_object = row
                            .try_get::<&str, _>(3)
                            .map_err(|_| SqlServerError::DatabaseError)?;
                        let error_number = row
                            .try_get::<i32, _>(4)
                            .map_err(|_| SqlServerError::DatabaseError)?;
                        let error_type = row
                            .try_get::<i32, _>(5)
                            .map_err(|_| SqlServerError::DatabaseError)?;
                        if dependency_is_cross_database(
                            &current_database,
                            referenced_database,
                            error_number,
                        ) || dependency_metadata_is_unsafe(
                            &current_database,
                            referenced_database,
                            referenced_schema,
                            referenced_object,
                            error_number,
                            error_type,
                        ) {
                            return Err(SqlServerError::QueryRejected);
                        }
                        if let (Some(schema), Some(object)) = (referenced_schema, referenced_object)
                        {
                            let object_rows = query_rows(
                                client,
                                "SELECT TOP (1) object_type FROM (
                                     SELECT o.type AS object_type
                                     FROM sys.objects AS o
                                     JOIN sys.schemas AS s ON s.schema_id = o.schema_id
                                     WHERE s.name = @P1 AND o.name = @P2
                                     UNION ALL
                                     SELECT 'SN'
                                     FROM sys.synonyms AS synonym
                                     JOIN sys.schemas AS synonym_schema
                                       ON synonym_schema.schema_id = synonym.schema_id
                                     WHERE synonym_schema.name = @P1 AND synonym.name = @P2
                                 ) AS dependency_types",
                                &[&schema, &object],
                                1,
                                limits.max_bytes,
                            )
                            .await?;
                            let object_type = object_rows
                                .first()
                                .ok_or(SqlServerError::QueryRejected)?
                                .try_get::<&str, _>(0)
                                .map_err(|_| SqlServerError::DatabaseError)?
                                .ok_or(SqlServerError::QueryRejected)?;
                            if unsupported_dependency_object_type(object_type) {
                                return Err(SqlServerError::QueryRejected);
                            }
                        }
                    }
                    let mut stream = client
                        .query(query_sql, &[])
                        .await
                        .map_err(|error| map_driver_error_for(error, DriverOperation::Query))?;
                    let columns = stream
                        .columns()
                        .await
                        .map_err(|error| map_driver_error_for(error, DriverOperation::Query))?
                        .ok_or(SqlServerError::DatabaseError)?
                        .iter()
                        .map(|column| {
                            logical_type(column.column_type()).map(|logical_type| {
                                Column::new(column.name().to_owned(), logical_type)
                            })
                        })
                        .collect::<Result<Vec<_>, _>>()?;
                    let mut rows = Vec::new();
                    let mut bytes = 0_u32;
                    let mut raw_budget = RowBudget::new(limits.max_rows as usize, limits.max_bytes);
                    let mut row_stream = stream.into_row_stream();
                    while let Some(result) = row_stream.next().await {
                        if rows.len() >= limits.max_rows as usize {
                            return Err(SqlServerError::ResultLimitExceeded);
                        }
                        let row = result
                            .map_err(|error| map_driver_error_for(error, DriverOperation::Query))?;
                        let raw_row_bytes = raw_row_size_bytes(&row, limits.max_bytes)?;
                        raw_budget.accept(raw_row_bytes)?;
                        let values = decode_row(&row)?;
                        bytes = bytes
                            .checked_add(
                                u32::try_from(
                                    serde_json::to_vec(&values)
                                        .map_err(|_| SqlServerError::DatabaseError)?
                                        .len(),
                                )
                                .map_err(|_| SqlServerError::ResultLimitExceeded)?,
                            )
                            .ok_or(SqlServerError::ResultLimitExceeded)?;
                        if bytes > limits.max_bytes {
                            return Err(SqlServerError::ResultLimitExceeded);
                        }
                        rows.push(values);
                    }
                    QueryResult::new(columns, rows).map_err(|_| SqlServerError::DatabaseError)
                })
            })
            .await
        })
    }

    fn open_client(
        &self,
        command_timeout: Duration,
    ) -> Result<(Runtime, TdsClient), SqlServerError> {
        let runtime = Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|_| SqlServerError::DatabaseError)?;
        let config = self.config.clone();
        let client = runtime.block_on(async move {
            let mut server = Config::new();
            server.host(&config.host);
            server.port(config.port);
            server.database(&config.database);
            server.authentication(AuthMethod::sql_server(&config.username, &config.password));
            apply_tls_mode(&mut server, config.tls_mode)?;
            server.readonly(true);
            server.handshake_timeout(Some(Duration::from_secs(5)));
            server.command_timeout(Some(command_timeout));
            let tcp = TcpStream::connect(server.get_addr())
                .await
                .map_err(|_| SqlServerError::ServerUnreachable)?;
            tcp.set_nodelay(true)
                .map_err(|_| SqlServerError::ServerUnreachable)?;
            Client::connect(server, tcp.compat_write())
                .await
                .map_err(map_driver_error)
        })?;
        Ok((runtime, client))
    }
}

fn apply_tls_mode(server: &mut Config, tls_mode: SqlServerTlsMode) -> Result<(), SqlServerError> {
    match tls_mode {
        SqlServerTlsMode::VerifyFull | SqlServerTlsMode::Require => {
            // Tiberius native-tls uses the OS trust store and hostname checks by default.
            server.encryption(EncryptionLevel::Required);
            Ok(())
        }
        SqlServerTlsMode::VerifyCa
        | SqlServerTlsMode::Disable
        | SqlServerTlsMode::TrustServerCertificate => Err(SqlServerError::TlsPolicyRejected),
    }
}

async fn in_read_only_transaction<T, F>(
    client: &mut TdsClient,
    limits: &QueryLimits,
    operation: F,
) -> Result<T, SqlServerError>
where
    F: for<'a> FnOnce(
        &'a mut TdsClient,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<T, SqlServerError>> + 'a>,
    >,
{
    let lock_timeout = format!(
        "SET LOCK_TIMEOUT {}; BEGIN TRANSACTION",
        limits.statement_timeout_ms
    );
    client
        .simple_query(&lock_timeout)
        .await
        .map_err(|error| map_driver_error_for(error, DriverOperation::Query))?
        .into_results()
        .await
        .map_err(|error| map_driver_error_for(error, DriverOperation::Query))?;
    let result = operation(client).await;
    let rollback = client
        .simple_query("IF @@TRANCOUNT > 0 ROLLBACK TRANSACTION")
        .await;
    if rollback.is_err() {
        return Err(SqlServerError::DatabaseError);
    }
    result
}

async fn query_rows(
    client: &mut TdsClient,
    sql: &str,
    params: &[&dyn ToSql],
    max_rows: usize,
    max_bytes: u32,
) -> Result<Vec<tiberius::Row>, SqlServerError> {
    let stream = client
        .query(sql, params)
        .await
        .map_err(|error| map_driver_error_for(error, DriverOperation::Query))?;
    let mut row_stream = stream.into_row_stream();
    let mut rows = Vec::new();
    let mut budget = RowBudget::new(max_rows, max_bytes);
    while let Some(result) = row_stream.next().await {
        let row = result.map_err(|error| map_driver_error_for(error, DriverOperation::Query))?;
        budget.accept(row_size_bytes(&row))?;
        if row.result_index() == 0 {
            rows.push(row);
        }
    }
    Ok(rows)
}

fn validate_cell_size(cell_bytes: usize, max_bytes: u32) -> Result<(), SqlServerError> {
    if cell_bytes > max_bytes as usize {
        Err(SqlServerError::ResultLimitExceeded)
    } else {
        Ok(())
    }
}

fn raw_row_size_bytes(row: &tiberius::Row, max_bytes: u32) -> Result<usize, SqlServerError> {
    let mut total = 0usize;
    for (_, data) in row.cells() {
        let cell_bytes = raw_cell_size_bytes(data);
        validate_cell_size(cell_bytes, max_bytes)?;
        total = total
            .checked_add(cell_bytes)
            .ok_or(SqlServerError::ResultLimitExceeded)?;
    }
    Ok(total)
}

fn raw_cell_size_bytes(data: &ColumnData<'_>) -> usize {
    match data {
        ColumnData::String(Some(value)) => value.len(),
        ColumnData::Binary(Some(value)) => value.len(),
        ColumnData::Xml(Some(value)) => value.as_ref().as_ref().len(),
        _ => std::mem::size_of_val(data),
    }
}

#[derive(Debug)]
struct RowBudget {
    max_rows: usize,
    max_bytes: u32,
    rows: usize,
    bytes: u32,
}

impl RowBudget {
    fn new(max_rows: usize, max_bytes: u32) -> Self {
        Self {
            max_rows,
            max_bytes,
            rows: 0,
            bytes: 0,
        }
    }

    fn accept(&mut self, row_bytes: usize) -> Result<(), SqlServerError> {
        let row_bytes =
            u32::try_from(row_bytes).map_err(|_| SqlServerError::ResultLimitExceeded)?;
        if self.rows >= self.max_rows {
            return Err(SqlServerError::ResultLimitExceeded);
        }
        let bytes = self
            .bytes
            .checked_add(row_bytes)
            .ok_or(SqlServerError::ResultLimitExceeded)?;
        if bytes > self.max_bytes {
            return Err(SqlServerError::ResultLimitExceeded);
        }
        self.rows += 1;
        self.bytes = bytes;
        Ok(())
    }
}

fn row_size_bytes(row: &tiberius::Row) -> usize {
    row.cells()
        .map(|(_, data)| {
            let inline_bytes = std::mem::size_of_val(data);
            match data {
                ColumnData::String(Some(value)) => inline_bytes.saturating_add(value.len()),
                ColumnData::Binary(Some(value)) => inline_bytes.saturating_add(value.len()),
                ColumnData::Xml(Some(value)) => {
                    inline_bytes.saturating_add(value.as_ref().as_ref().len())
                }
                _ => inline_bytes,
            }
        })
        .fold(0usize, usize::saturating_add)
}

fn text_cell(row: &tiberius::Row, index: usize) -> Result<String, SqlServerError> {
    row.try_get::<&str, _>(index)
        .map_err(|_| SqlServerError::DatabaseError)?
        .map(str::to_owned)
        .ok_or(SqlServerError::DatabaseError)
}

fn decode_row(row: &tiberius::Row) -> Result<Vec<SqlValue>, SqlServerError> {
    row.columns()
        .iter()
        .enumerate()
        .map(|(index, column)| {
            let data = row
                .get_column_data(index)
                .map_err(|_| SqlServerError::DatabaseError)?;
            decode_value(data, column.column_type())
        })
        .collect()
}

fn decode_value(
    data: &ColumnData<'_>,
    column_type: ColumnType,
) -> Result<SqlValue, SqlServerError> {
    Ok(match data {
        ColumnData::U8(value) => {
            value.map_or(SqlValue::Null, |value| SqlValue::Unsigned(value as u64))
        }
        ColumnData::I16(value) => {
            value.map_or(SqlValue::Null, |value| SqlValue::Signed(value as i64))
        }
        ColumnData::I32(value) => {
            value.map_or(SqlValue::Null, |value| SqlValue::Signed(value as i64))
        }
        ColumnData::I64(value) => value.map_or(SqlValue::Null, SqlValue::Signed),
        ColumnData::F32(value) => value.map_or(SqlValue::Null, |value| SqlValue::Float {
            bits: value.to_bits() as u64,
        }),
        ColumnData::F64(value) => value.map_or(SqlValue::Null, |value| SqlValue::Float {
            bits: value.to_bits(),
        }),
        ColumnData::Bit(value) => value.map_or(SqlValue::Null, SqlValue::Bool),
        ColumnData::String(value) => value
            .as_ref()
            .map_or(SqlValue::Null, |value| SqlValue::Text(value.to_string())),
        ColumnData::Guid(value) => {
            value.map_or(SqlValue::Null, |value| SqlValue::Uuid(value.to_string()))
        }
        ColumnData::Binary(value) => value
            .as_ref()
            .map_or(SqlValue::Null, |value| SqlValue::Bytes(value.to_vec())),
        ColumnData::Numeric(value) => value
            .as_ref()
            .map_or(SqlValue::Null, |value| SqlValue::Decimal(value.to_string())),
        ColumnData::DateTime(value) => value.as_ref().map_or(SqlValue::Null, |value| {
            SqlValue::Timestamp(format!("{value:?}"))
        }),
        ColumnData::SmallDateTime(value) => value.as_ref().map_or(SqlValue::Null, |value| {
            SqlValue::Timestamp(format!("{value:?}"))
        }),
        ColumnData::Time(value) => value.as_ref().map_or(SqlValue::Null, |value| {
            SqlValue::Timestamp(format!("{value:?}"))
        }),
        ColumnData::Date(value) => value
            .as_ref()
            .map_or(SqlValue::Null, |value| SqlValue::Date(format!("{value:?}"))),
        ColumnData::DateTime2(value) => value.as_ref().map_or(SqlValue::Null, |value| {
            SqlValue::Timestamp(format!("{value:?}"))
        }),
        ColumnData::DateTimeOffset(value) => value.as_ref().map_or(SqlValue::Null, |value| {
            SqlValue::Timestamptz(format!("{value:?}"))
        }),
        _ => return Err(SqlServerError::UnsupportedType(format!("{column_type:?}"))),
    })
}

fn logical_type(column_type: ColumnType) -> Result<LogicalType, SqlServerError> {
    match column_type {
        ColumnType::Bit | ColumnType::Bitn => Ok(LogicalType::Boolean),
        ColumnType::Int1 => Ok(LogicalType::Integer),
        ColumnType::Int2 | ColumnType::Int4 => Ok(LogicalType::Integer),
        ColumnType::Int8 | ColumnType::Intn => Ok(LogicalType::Bigint),
        ColumnType::Float4 | ColumnType::Floatn => Ok(LogicalType::Float),
        ColumnType::Float8 => Ok(LogicalType::Float),
        ColumnType::Money | ColumnType::Money4 | ColumnType::Numericn | ColumnType::Decimaln => {
            Ok(LogicalType::Numeric)
        }
        ColumnType::BigVarChar
        | ColumnType::BigChar
        | ColumnType::NVarchar
        | ColumnType::NChar
        | ColumnType::Text
        | ColumnType::NText => Ok(LogicalType::Text),
        ColumnType::BigVarBin | ColumnType::BigBinary | ColumnType::Image => {
            Ok(LogicalType::Binary)
        }
        ColumnType::Daten => Ok(LogicalType::Date),
        ColumnType::Datetime
        | ColumnType::Datetime4
        | ColumnType::Datetimen
        | ColumnType::Datetime2 => Ok(LogicalType::Timestamp),
        ColumnType::DatetimeOffsetn => Ok(LogicalType::Timestamptz),
        ColumnType::Guid => Ok(LogicalType::Uuid),
        _ => Err(SqlServerError::UnsupportedType(format!("{column_type:?}"))),
    }
}

fn validate_limits(limits: &QueryLimits) -> Result<(), SqlServerError> {
    limits
        .validate()
        .map_err(|error| SqlServerError::InvalidLimits(error.to_string()))
}

fn timeout(limits: &QueryLimits) -> Duration {
    Duration::from_millis(u64::from(limits.statement_timeout_ms))
}

fn validate_config(config: &SqlServerConfig) -> Result<(), SqlServerError> {
    for (value, field) in [
        (&config.host, "host"),
        (&config.database, "database"),
        (&config.username, "username"),
    ] {
        if value.trim().is_empty() || value.len() > 256 || value.chars().any(char::is_control) {
            return Err(SqlServerError::InvalidProfile(field));
        }
    }
    if config.host.chars().any(char::is_whitespace)
        || config.host.contains('/')
        || config.host.contains('\\')
    {
        return Err(SqlServerError::InvalidProfile("host"));
    }
    if config.port == 0 {
        return Err(SqlServerError::InvalidProfile("port"));
    }
    if matches!(
        config.tls_mode,
        SqlServerTlsMode::Disable | SqlServerTlsMode::TrustServerCertificate
    ) {
        return Err(SqlServerError::TlsPolicyRejected);
    }
    Ok(())
}

#[derive(Clone, Copy)]
enum DriverOperation {
    Connect,
    Query,
}

fn map_driver_error(error: tiberius::error::Error) -> SqlServerError {
    map_driver_error_for(error, DriverOperation::Connect)
}

fn map_driver_error_for(
    error: tiberius::error::Error,
    operation: DriverOperation,
) -> SqlServerError {
    if std::env::var_os("SQLSERVER_DEBUG").is_some() {
        eprintln!("SQL Server driver error: {error:?}");
    }
    match error {
        tiberius::error::Error::Tls(_) => SqlServerError::TlsFailed,
        tiberius::error::Error::Server(error) if error.code() == 18456 => {
            SqlServerError::AuthenticationFailed
        }
        tiberius::error::Error::Io { kind, .. } => map_driver_io_error(kind, operation),
        _ => SqlServerError::DatabaseError,
    }
}

fn map_driver_io_error(kind: std::io::ErrorKind, operation: DriverOperation) -> SqlServerError {
    if kind == std::io::ErrorKind::TimedOut && matches!(operation, DriverOperation::Query) {
        return SqlServerError::QueryTimeout;
    }
    match kind {
        std::io::ErrorKind::TimedOut
        | std::io::ErrorKind::ConnectionRefused
        | std::io::ErrorKind::ConnectionReset
        | std::io::ErrorKind::NotConnected
        | std::io::ErrorKind::UnexpectedEof => SqlServerError::ServerUnreachable,
        _ => SqlServerError::DatabaseError,
    }
}

pub fn validate_read_only_tsql(sql: &str) -> Result<(), SqlServerError> {
    let normalized = normalize_tsql(sql)?;
    let tokens = tsql_tokens(&normalized);
    if !matches!(tokens.first(), Some(&"select" | &"with")) {
        return Err(SqlServerError::QueryRejected);
    }
    const FORBIDDEN: &[&str] = &[
        "insert",
        "update",
        "delete",
        "merge",
        "drop",
        "alter",
        "truncate",
        "grant",
        "revoke",
        "into",
        "exec",
        "execute",
        "xp_cmdshell",
        "openquery",
        "openrowset",
        "opendatasource",
        "waitfor",
        "bulk",
        "backup",
        "restore",
    ];
    if tokens.iter().any(|token| FORBIDDEN.contains(token)) {
        return Err(SqlServerError::QueryRejected);
    }
    if has_cross_database_reference(&tokens) {
        return Err(SqlServerError::QueryRejected);
    }
    if has_unsupported_function_call(&tokens) {
        return Err(SqlServerError::QueryRejected);
    }
    Ok(())
}

fn normalize_tsql(sql: &str) -> Result<String, SqlServerError> {
    let mut output = String::with_capacity(sql.len());
    let mut chars = sql.chars().peekable();
    let mut saw_separator = false;
    let mut has_content = false;
    while let Some(character) = chars.next() {
        match character {
            '\'' => {
                let mut closed = false;
                while let Some(literal) = chars.next() {
                    if literal == '\'' {
                        if chars.peek() == Some(&'\'') {
                            chars.next();
                        } else {
                            closed = true;
                            break;
                        }
                    }
                }
                if !closed {
                    return Err(SqlServerError::QueryRejected);
                }
                output.push(' ');
            }
            '-' if chars.peek() == Some(&'-') => {
                chars.next();
                for comment in chars.by_ref() {
                    if comment == '\n' {
                        break;
                    }
                }
                output.push(' ');
            }
            '/' if chars.peek() == Some(&'*') => {
                chars.next();
                let mut closed = false;
                while let Some(comment) = chars.next() {
                    if comment == '*' && chars.peek() == Some(&'/') {
                        chars.next();
                        closed = true;
                        break;
                    }
                }
                if !closed {
                    return Err(SqlServerError::QueryRejected);
                }
                output.push(' ');
            }
            ';' => {
                if !has_content {
                    return Err(SqlServerError::QueryRejected);
                }
                saw_separator = true;
                has_content = false;
                output.push(' ');
            }
            _ => {
                if !character.is_whitespace() {
                    if saw_separator {
                        return Err(SqlServerError::QueryRejected);
                    }
                    has_content = true;
                }
                output.push(character.to_ascii_lowercase());
            }
        }
    }
    if output.trim().is_empty() || !has_content && !saw_separator {
        return Err(SqlServerError::QueryRejected);
    }
    Ok(output)
}

fn has_cross_database_reference(tokens: &[&str]) -> bool {
    tokens.windows(4).any(|window| {
        !window[0].is_empty() && window[1] == "." && window[2] == "." && !window[3].is_empty()
    }) || tokens.windows(5).any(|window| {
        !window[0].is_empty()
            && window[1] == "."
            && !window[2].is_empty()
            && window[3] == "."
            && !window[4].is_empty()
    })
}

fn dependency_is_cross_database(
    current_database: &str,
    referenced_database: Option<&str>,
    error_number: Option<i32>,
) -> bool {
    error_number.is_some_and(|number| number != 0)
        || referenced_database
            .is_some_and(|database| !database.eq_ignore_ascii_case(current_database))
}

fn dependency_metadata_is_unsafe(
    current_database: &str,
    referenced_database: Option<&str>,
    referenced_schema: Option<&str>,
    referenced_object: Option<&str>,
    error_number: Option<i32>,
    error_type: Option<i32>,
) -> bool {
    if error_number.is_some_and(|number| number != 0) || error_type.is_some_and(|kind| kind != 0) {
        return true;
    }
    if referenced_database.is_none() && (referenced_schema.is_some() || referenced_object.is_some())
    {
        return true;
    }
    if referenced_database.is_some_and(|database| !database.eq_ignore_ascii_case(current_database))
    {
        return true;
    }
    referenced_schema.is_some() != referenced_object.is_some()
}

fn unsupported_dependency_object_type(object_type: &str) -> bool {
    !matches!(object_type, "U" | "S")
}

fn has_unsupported_function_call(tokens: &[&str]) -> bool {
    const SAFE_FUNCTIONS: &[&str] = &[
        "abs",
        "avg",
        "cast",
        "coalesce",
        "concat",
        "convert",
        "count",
        "dateadd",
        "datediff",
        "day",
        "getdate",
        "isnull",
        "len",
        "lower",
        "ltrim",
        "max",
        "min",
        "month",
        "nullif",
        "replace",
        "round",
        "rtrim",
        "sum",
        "substring",
        "try_convert",
        "upper",
        "year",
    ];
    tokens.windows(2).any(|window| {
        window[1] == "("
            && !SAFE_FUNCTIONS.contains(&window[0])
            && !matches!(
                window[0],
                "over"
                    | "in"
                    | "exists"
                    | "not"
                    | "and"
                    | "or"
                    | "as"
                    | "from"
                    | "join"
                    | "where"
                    | "select"
                    | "values"
                    | "case"
                    | "when"
                    | "top"
            )
    })
}

fn read_only_from_permissions(snapshot: PermissionSnapshot) -> bool {
    [
        snapshot.server_escalation,
        snapshot.database_escalation,
        snapshot.database_write,
        snapshot.object_write,
    ]
    .into_iter()
    .all(|write_capable| write_capable == Some(false))
}

fn tsql_tokens(sql: &str) -> Vec<&str> {
    let mut tokens = Vec::new();
    let mut start = None;
    for (index, character) in sql.char_indices() {
        if character.is_ascii_alphanumeric() || character == '_' {
            start.get_or_insert(index);
        } else {
            if let Some(start) = start.take() {
                tokens.push(&sql[start..index]);
            }
            if matches!(character, '.' | '(' | ')') {
                tokens.push(&sql[index..index + 1]);
            }
        }
    }
    if let Some(start) = start {
        tokens.push(&sql[start..]);
    }
    tokens
}

#[cfg(test)]
mod tests {
    use super::{
        DriverOperation, PermissionSnapshot, RowBudget, SqlServerError,
        dependency_is_cross_database, map_driver_io_error, read_only_from_permissions,
        validate_cell_size,
    };

    #[test]
    fn writer_role_is_not_reported_as_read_only() {
        assert!(!read_only_from_permissions(PermissionSnapshot {
            server_escalation: Some(false),
            database_escalation: Some(true),
            database_write: Some(false),
            object_write: Some(false),
        }));
    }

    #[test]
    fn unknown_permission_result_fails_closed() {
        assert!(!read_only_from_permissions(PermissionSnapshot {
            server_escalation: Some(false),
            database_escalation: Some(false),
            database_write: None,
            object_write: Some(false),
        }));
    }

    #[test]
    fn dependency_metadata_rejects_errors_and_other_databases() {
        assert!(dependency_is_cross_database(
            "practice",
            Some("other"),
            None
        ));
        assert!(dependency_is_cross_database(
            "practice",
            Some("PRACTICE"),
            Some(208)
        ));
        assert!(dependency_is_cross_database("practice", None, Some(208)));
        assert!(!dependency_is_cross_database(
            "practice",
            Some("practice"),
            None
        ));
        assert!(!dependency_is_cross_database("practice", None, Some(0)));
    }

    #[test]
    fn dependency_metadata_rejects_unresolved_indirect_sources() {
        assert!(super::dependency_metadata_is_unsafe(
            "practice",
            Some("practice"),
            Some("dbo"),
            None,
            Some(0),
            Some(0),
        ));
        assert!(super::dependency_metadata_is_unsafe(
            "practice",
            None,
            Some("dbo"),
            Some("hidden_view"),
            Some(0),
            Some(0),
        ));
        assert!(!super::dependency_metadata_is_unsafe(
            "practice",
            Some("practice"),
            Some("dbo"),
            Some("employees"),
            Some(0),
            Some(0),
        ));
        assert!(super::unsupported_dependency_object_type("V"));
        assert!(super::unsupported_dependency_object_type("SN"));
        assert!(super::unsupported_dependency_object_type("IF"));
        assert!(!super::unsupported_dependency_object_type("U"));
        assert_eq!(
            super::validate_read_only_tsql("SELECT dbo.hidden_fn(value) FROM dbo.items"),
            Err(super::SqlServerError::QueryRejected)
        );
    }

    #[test]
    fn row_budget_rejects_rows_and_bytes_before_accumulation() {
        let mut budget = RowBudget::new(2, 10);
        assert_eq!(budget.accept(4), Ok(()));
        assert_eq!(budget.accept(6), Ok(()));
        assert_eq!(budget.accept(1), Err(SqlServerError::ResultLimitExceeded));

        let mut budget = RowBudget::new(2, 10);
        assert_eq!(budget.accept(11), Err(SqlServerError::ResultLimitExceeded));
    }

    #[test]
    fn raw_cell_size_rejects_oversized_cells_before_decode() {
        assert_eq!(validate_cell_size(10, 10), Ok(()));
        assert_eq!(
            validate_cell_size(11, 10),
            Err(SqlServerError::ResultLimitExceeded)
        );
    }

    #[test]
    fn timed_out_connection_and_query_are_distinguished() {
        assert_eq!(
            map_driver_io_error(std::io::ErrorKind::TimedOut, DriverOperation::Connect),
            super::SqlServerError::ServerUnreachable
        );
        assert_eq!(
            map_driver_io_error(std::io::ErrorKind::TimedOut, DriverOperation::Query),
            super::SqlServerError::QueryTimeout
        );
    }
}
