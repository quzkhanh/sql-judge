use fallible_iterator::FallibleIterator;
use judge_core::{Column, LogicalType, QueryLimits, QueryResult, SqlValue};
use postgres::{
    Client, NoTls, Row,
    types::{ToSql, Type},
};
use std::time::{SystemTime, UNIX_EPOCH};
use thiserror::Error;

mod orchestrator;

pub use orchestrator::{JudgeCase, SubmissionReport, TestOutcome, judge_submission};

#[derive(Debug, Error, PartialEq, Eq)]
pub enum QueryExecutionError {
    #[error("query violates the read-only SQL policy")]
    PolicyViolation,
    #[error("query limits are invalid: {0}")]
    InvalidLimits(String),
    #[error("query has invalid SQL syntax")]
    SyntaxError,
    #[error("query exceeded its statement deadline")]
    TimeLimitExceeded,
    #[error("query exceeded the result limit")]
    ResultLimitExceeded,
    #[error("database execution failed")]
    RuntimeError,
    #[error("unsupported PostgreSQL result type: {0}")]
    UnsupportedType(String),
    #[error("could not connect to the managed PostgreSQL instance")]
    ConnectionError,
    #[error("managed namespace ownership marker does not match")]
    OwnershipVerificationFailed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ManagedEnvironment {
    environment_id: String,
    instance_id: String,
    namespace: String,
    ownership_nonce: String,
}

impl ManagedEnvironment {
    pub fn new(
        environment_id: impl Into<String>,
        instance_id: impl Into<String>,
        namespace: impl Into<String>,
        ownership_nonce: impl Into<String>,
    ) -> Result<Self, QueryExecutionError> {
        let environment = Self {
            environment_id: environment_id.into(),
            instance_id: instance_id.into(),
            namespace: namespace.into(),
            ownership_nonce: ownership_nonce.into(),
        };
        if environment.environment_id.trim().is_empty()
            || environment.instance_id.trim().is_empty()
            || environment.ownership_nonce.trim().is_empty()
            || !is_safe_namespace(&environment.namespace)
        {
            return Err(QueryExecutionError::PolicyViolation);
        }
        Ok(environment)
    }
}

pub struct PostgresSession {
    client: Client,
}

pub struct ManagedPostgresSession {
    session: PostgresSession,
}

impl ManagedPostgresSession {
    pub fn connect(
        connection_string: &str,
        expected_instance_id: &str,
    ) -> Result<Self, QueryExecutionError> {
        if expected_instance_id.trim().is_empty() {
            return Err(QueryExecutionError::PolicyViolation);
        }
        let mut session = PostgresSession::connect(connection_string)?;
        let marker_exists = session
            .client
            .query_one(
                "SELECT EXISTS (
                     SELECT 1
                     FROM pg_catalog.pg_class c
                     JOIN pg_catalog.pg_namespace n ON n.oid = c.relnamespace
                     WHERE n.nspname = 'public' AND c.relname = '__sql_judge_runtime'
                       AND c.relkind IN ('r', 'p')
                 )",
                &[],
            )
            .map_err(map_database_error)?
            .get::<_, bool>(0);
        if !marker_exists {
            return Err(QueryExecutionError::OwnershipVerificationFailed);
        }
        let marker = session
            .client
            .query_opt(
                "SELECT r.instance_id
                 FROM public.__sql_judge_runtime r
                 JOIN pg_catalog.pg_class c ON c.relname = '__sql_judge_runtime'
                 JOIN pg_catalog.pg_namespace n ON n.oid = c.relnamespace
                 JOIN pg_catalog.pg_user u ON u.usesysid = c.relowner
                 WHERE n.nspname = 'public' AND c.relname = '__sql_judge_runtime'
                   AND u.usename = current_user
                 LIMIT 1",
                &[],
            )
            .map_err(map_database_error)?;
        if marker.is_none_or(|row| row.get::<_, String>(0) != expected_instance_id) {
            return Err(QueryExecutionError::OwnershipVerificationFailed);
        }
        Ok(Self { session })
    }

    pub fn with_managed_fixture<T, F>(
        &mut self,
        runner_role: &str,
        schema_sql: &str,
        fixture_sql: &str,
        operation: F,
    ) -> Result<T, QueryExecutionError>
    where
        F: FnOnce(&mut PostgresSession) -> Result<T, QueryExecutionError>,
    {
        self.session
            .with_managed_fixture(runner_role, schema_sql, fixture_sql, operation)
    }
}

impl PostgresSession {
    pub fn connect(connection_string: &str) -> Result<Self, QueryExecutionError> {
        if !is_local_connection_string(connection_string) {
            return Err(QueryExecutionError::PolicyViolation);
        }
        Client::connect(connection_string, NoTls)
            .map(|client| Self { client })
            .map_err(|_| QueryExecutionError::ConnectionError)
    }

    pub fn execute(
        &mut self,
        sql: &str,
        limits: QueryLimits,
    ) -> Result<QueryResult, QueryExecutionError> {
        limits
            .validate()
            .map_err(|error| QueryExecutionError::InvalidLimits(error.to_string()))?;
        validate_read_only_sql(sql)?;

        self.client
            .batch_execute(&format!(
                "SET statement_timeout = {}",
                limits.statement_timeout_ms
            ))
            .map_err(map_database_error)?;

        let statement = self.client.prepare(sql).map_err(map_database_error)?;
        let columns = statement
            .columns()
            .iter()
            .map(|column| {
                logical_type(column.type_())
                    .map(|logical_type| Column::new(column.name(), logical_type))
            })
            .collect::<Result<Vec<_>, _>>()?;
        let mut rows = self
            .client
            .query_raw(&statement, std::iter::empty::<&(dyn ToSql + Sync)>())
            .map_err(map_database_error)?;
        let mut values = Vec::with_capacity(limits.max_rows as usize);
        let mut result_bytes = 0_u32;
        let mut row_count = 0_usize;

        while let Some(row) = rows.next().map_err(map_database_error)? {
            if row_count >= limits.max_rows as usize {
                return Err(QueryExecutionError::ResultLimitExceeded);
            }
            let row_values = decode_row(&row)?;
            result_bytes = result_bytes
                .checked_add(
                    serde_json::to_vec(&row_values)
                        .map_err(|_| QueryExecutionError::RuntimeError)?
                        .len() as u32,
                )
                .ok_or(QueryExecutionError::ResultLimitExceeded)?;
            if result_bytes > limits.max_bytes {
                return Err(QueryExecutionError::ResultLimitExceeded);
            }
            values.push(row_values);
            row_count += 1;
        }

        QueryResult::new(columns, values).map_err(|_| QueryExecutionError::RuntimeError)
    }

    pub fn with_managed_fixture<T, F>(
        &mut self,
        runner_role: &str,
        schema_sql: &str,
        fixture_sql: &str,
        operation: F,
    ) -> Result<T, QueryExecutionError>
    where
        F: FnOnce(&mut Self) -> Result<T, QueryExecutionError>,
    {
        if !is_safe_role_name(runner_role) {
            return Err(QueryExecutionError::PolicyViolation);
        }
        validate_setup_sql(schema_sql)?;
        validate_setup_sql(fixture_sql)?;
        let runner_is_constrained = self
            .client
            .query_opt(
                "SELECT NOT rolsuper AND NOT rolcreaterole AND NOT rolcreatedb
                    AND NOT rolinherit AND NOT rolbypassrls AND NOT rolreplication
                 FROM pg_catalog.pg_roles WHERE rolname = $1",
                &[&runner_role],
            )
            .map_err(map_database_error)?
            .is_some_and(|row| row.get::<_, bool>(0));
        if !runner_is_constrained {
            return Err(QueryExecutionError::PolicyViolation);
        }

        let fixture_suffix = unique_fixture_suffix();
        let fixture_schema = format!("sj_fixture_{fixture_suffix}");
        let fixture_role = format!("sj_fixture_{fixture_suffix}");
        let schema = quote_identifier(&fixture_schema);
        let role = quote_identifier(&fixture_role);
        let runner = quote_identifier(runner_role);

        self.client
            .batch_execute("BEGIN")
            .map_err(|_| QueryExecutionError::RuntimeError)?;
        let setup = (|| {
            self.client
                .batch_execute(&format!(
                    "REVOKE ALL ON SCHEMA public FROM PUBLIC;
                     REVOKE ALL ON ALL TABLES IN SCHEMA public FROM PUBLIC;
                     CREATE ROLE {role} NOLOGIN NOSUPERUSER NOCREATEDB NOCREATEROLE NOINHERIT;
                     CREATE SCHEMA {schema} AUTHORIZATION {role};
                     SET LOCAL search_path TO {schema}, pg_catalog;
                     SET LOCAL ROLE {role}"
                ))
                .map_err(|_| QueryExecutionError::RuntimeError)?;
            self.client
                .batch_execute(schema_sql)
                .map_err(|_| QueryExecutionError::RuntimeError)?;
            self.client
                .batch_execute(fixture_sql)
                .map_err(|_| QueryExecutionError::RuntimeError)?;
            self.client
                .batch_execute(&format!(
                    "RESET ROLE;
                     GRANT USAGE ON SCHEMA {schema} TO {runner};
                     GRANT SELECT ON ALL TABLES IN SCHEMA {schema} TO {runner};
                     SET LOCAL ROLE {runner}"
                ))
                .map_err(|_| QueryExecutionError::RuntimeError)?;
            operation(self)
        })();
        let rollback = self.client.batch_execute("ROLLBACK");

        if rollback.is_err() {
            Err(QueryExecutionError::RuntimeError)
        } else {
            setup
        }
    }

    pub fn provision_managed_namespace(
        &mut self,
        environment: &ManagedEnvironment,
        runner_role: &str,
    ) -> Result<(), QueryExecutionError> {
        if !is_safe_role_name(runner_role) {
            return Err(QueryExecutionError::PolicyViolation);
        }

        let schema = quote_identifier(&environment.namespace);
        let marker = format!("{schema}.\"__sql_judge_ownership\"");
        let role = quote_identifier(runner_role);
        let mut transaction = self
            .client
            .transaction()
            .map_err(|_| QueryExecutionError::RuntimeError)?;
        let result = (|| {
            let runner_is_constrained = transaction
                .query_opt(
                    "SELECT NOT rolsuper AND NOT rolcreaterole AND NOT rolcreatedb
                        AND NOT rolinherit AND NOT rolbypassrls AND NOT rolreplication
                     FROM pg_catalog.pg_roles WHERE rolname = $1",
                    &[&runner_role],
                )
                .map_err(map_database_error)?
                .is_some_and(|row| row.get::<_, bool>(0));
            if !runner_is_constrained {
                return Err(QueryExecutionError::PolicyViolation);
            }
            let schema_exists = transaction
                .query_one(
                    "SELECT EXISTS (
                         SELECT 1 FROM pg_catalog.pg_namespace WHERE nspname = $1
                     )",
                    &[&environment.namespace],
                )
                .map_err(map_database_error)?
                .get::<_, bool>(0);
            let marker_exists = transaction
                .query_one(
                    "SELECT to_regclass($1) IS NOT NULL",
                    &[&format!(
                        "{}.{}",
                        environment.namespace, "__sql_judge_ownership"
                    )],
                )
                .map_err(map_database_error)?
                .get::<_, bool>(0);
            if schema_exists && !marker_exists {
                return Err(QueryExecutionError::OwnershipVerificationFailed);
            }
            if schema_exists {
                let marker_owned = transaction
                    .query_one(
                        "SELECT n.nspowner = u.usesysid AND c.relowner = u.usesysid
                         FROM pg_catalog.pg_namespace n
                         JOIN pg_catalog.pg_class c ON c.relnamespace = n.oid
                         JOIN pg_catalog.pg_user u ON u.usename = current_user
                         WHERE n.nspname = $1 AND c.relname = $2
                           AND c.relkind IN ('r', 'p')",
                        &[&environment.namespace, &"__sql_judge_ownership"],
                    )
                    .map_err(map_database_error)?
                    .get::<_, bool>(0);
                if !marker_owned {
                    return Err(QueryExecutionError::OwnershipVerificationFailed);
                }
            }
            transaction
                .batch_execute(&format!(
                    "CREATE SCHEMA IF NOT EXISTS {schema};
                     REVOKE ALL ON SCHEMA {schema} FROM PUBLIC;
                     CREATE TABLE IF NOT EXISTS {marker} (
                         marker_key text PRIMARY KEY CHECK (marker_key = 'environment'),
                         environment_id text NOT NULL,
                         instance_id text NOT NULL,
                         ownership_nonce text NOT NULL
                     )"
                ))
                .map_err(map_database_error)?;

            let existing = transaction
                .query_opt(
                    &format!(
                        "SELECT environment_id, instance_id, ownership_nonce FROM {marker} WHERE marker_key = 'environment'"
                    ),
                    &[],
                )
                .map_err(map_database_error)?;
            if let Some(row) = existing {
                let matches = row.get::<_, String>(0) == environment.environment_id
                    && row.get::<_, String>(1) == environment.instance_id
                    && row.get::<_, String>(2) == environment.ownership_nonce;
                if !matches {
                    return Err(QueryExecutionError::OwnershipVerificationFailed);
                }
            } else {
                transaction
                    .execute(
                        &format!(
                            "INSERT INTO {marker} (marker_key, environment_id, instance_id, ownership_nonce)
                             VALUES ('environment', $1, $2, $3)"
                        ),
                        &[
                            &environment.environment_id,
                            &environment.instance_id,
                            &environment.ownership_nonce,
                        ],
                    )
                    .map_err(map_database_error)?;
            }

            transaction
                .batch_execute(&format!(
                    "GRANT USAGE ON SCHEMA {schema} TO {role};
                     GRANT SELECT ON ALL TABLES IN SCHEMA {schema} TO {role};
                     REVOKE ALL ON TABLE {marker} FROM {role}, PUBLIC"
                ))
                .map_err(map_database_error)?;
            Ok(())
        })();

        match result {
            Ok(()) => transaction
                .commit()
                .map_err(|_| QueryExecutionError::RuntimeError),
            Err(error) => {
                let _ = transaction.rollback();
                Err(error)
            }
        }
    }

    pub fn provision_managed_namespace_with_schema(
        &mut self,
        environment: &ManagedEnvironment,
        runner_role: &str,
        schema_sql: &str,
    ) -> Result<(), QueryExecutionError> {
        validate_setup_sql(schema_sql)?;
        self.provision_managed_namespace(environment, runner_role)?;

        let schema = quote_identifier(&environment.namespace);
        self.client
            .batch_execute("BEGIN")
            .map_err(|_| QueryExecutionError::RuntimeError)?;
        let setup = self
            .client
            .batch_execute(&format!(
                "SET LOCAL search_path TO {schema}, pg_catalog;\n{schema_sql}"
            ))
            .map_err(|_| QueryExecutionError::RuntimeError);
        let rollback = self.client.batch_execute("COMMIT");
        if rollback.is_err() {
            Err(QueryExecutionError::RuntimeError)
        } else {
            setup
        }
    }
}

fn is_safe_role_name(role: &str) -> bool {
    !role.is_empty()
        && role
            .chars()
            .next()
            .is_some_and(|character| character.is_ascii_lowercase() || character == '_')
        && role.chars().all(|character| {
            character.is_ascii_lowercase() || character.is_ascii_digit() || character == '_'
        })
}

fn is_local_connection_string(connection_string: &str) -> bool {
    if connection_string.starts_with("postgres://")
        || connection_string.starts_with("postgresql://")
    {
        return false;
    }
    let mut saw_target = false;
    for part in connection_string.split_whitespace() {
        let Some(host) = part
            .strip_prefix("host=")
            .or_else(|| part.strip_prefix("hostaddr="))
        else {
            continue;
        };
        saw_target = true;
        if host.starts_with('/') {
            continue;
        }
        if !matches!(host, "127.0.0.1" | "::1" | "localhost") {
            return false;
        }
    }
    saw_target
}

fn unique_fixture_suffix() -> String {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.as_nanos());
    format!("{}_{}", std::process::id(), timestamp)
}

fn is_safe_namespace(namespace: &str) -> bool {
    namespace.len() <= 63
        && namespace.starts_with("sj_")
        && namespace
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
}

fn quote_identifier(identifier: &str) -> String {
    format!("\"{identifier}\"")
}

pub fn validate_read_only_sql(sql: &str) -> Result<(), QueryExecutionError> {
    let normalized = strip_literals_comments_and_check_semicolons(sql)?;
    let tokens: Vec<&str> = normalized
        .split(|character: char| !character.is_ascii_alphanumeric() && character != '_')
        .filter(|token| !token.is_empty())
        .collect();

    const FORBIDDEN: &[&str] = &[
        "insert",
        "update",
        "delete",
        "drop",
        "alter",
        "create",
        "truncate",
        "grant",
        "revoke",
        "copy",
        "call",
        "execute",
        "do",
        "listen",
        "notify",
        "vacuum",
        "analyze",
        "pg_sleep",
        "pg_sleep_for",
        "dblink",
        "lo_import",
        "lo_export",
        "nextval",
        "set_config",
        "pg_read_file",
        "pg_ls_dir",
    ];
    if !matches!(tokens.first(), Some(&"select" | &"with")) {
        if tokens
            .first()
            .is_some_and(|token| FORBIDDEN.contains(token))
        {
            return Err(QueryExecutionError::PolicyViolation);
        }
        return Err(QueryExecutionError::SyntaxError);
    }
    if tokens.iter().any(|token| FORBIDDEN.contains(token)) {
        return Err(QueryExecutionError::PolicyViolation);
    }

    Ok(())
}

pub fn validate_setup_sql(sql: &str) -> Result<(), QueryExecutionError> {
    const FORBIDDEN: &[&str] = &[
        "alter",
        "analyze",
        "call",
        "copy",
        "database",
        "dblink",
        "delete",
        "do",
        "drop",
        "execute",
        "function",
        "grant",
        "listen",
        "nextval",
        "notify",
        "pg_ls_dir",
        "pg_read_file",
        "pg_sleep",
        "pg_sleep_for",
        "public",
        "revoke",
        "role",
        "schema",
        "sequence",
        "select",
        "set",
        "trigger",
        "truncate",
        "update",
        "vacuum",
    ];
    for statement in split_setup_statements(sql)? {
        let tokens: Vec<&str> = statement
            .split(|character: char| !character.is_ascii_alphanumeric() && character != '_')
            .filter(|token| !token.is_empty())
            .collect();
        if tokens.is_empty()
            || tokens.iter().any(|token| {
                FORBIDDEN.contains(token) || token.starts_with("pg_") || token.starts_with("lo_")
            })
            || !matches!(
                tokens.as_slice(),
                ["create", "table", ..] | ["create", "index", ..] | ["insert", "into", ..]
            )
        {
            return Err(QueryExecutionError::PolicyViolation);
        }
    }
    Ok(())
}

fn split_setup_statements(sql: &str) -> Result<Vec<String>, QueryExecutionError> {
    let mut statements = Vec::new();
    let mut current = String::new();
    let mut characters = sql.chars().peekable();
    let mut in_single_quote = false;
    let mut in_double_quote = false;
    let mut in_line_comment = false;
    let mut in_block_comment = false;

    while let Some(character) = characters.next() {
        if in_line_comment {
            if character == '\n' {
                in_line_comment = false;
            }
            current.push(' ');
            continue;
        }
        if in_block_comment {
            if character == '*' && characters.peek() == Some(&'/') {
                characters.next();
                in_block_comment = false;
            }
            current.push(' ');
            continue;
        }
        if in_single_quote {
            if character == '\'' {
                if characters.peek() == Some(&'\'') {
                    characters.next();
                } else {
                    in_single_quote = false;
                }
            }
            current.push(' ');
            continue;
        }
        if in_double_quote {
            if character == '"' {
                in_double_quote = false;
            } else {
                current.push(character.to_ascii_lowercase());
            }
            continue;
        }

        match character {
            '\'' => {
                in_single_quote = true;
                current.push(' ');
            }
            '"' => {
                in_double_quote = true;
                current.push(' ');
            }
            '-' if characters.peek() == Some(&'-') => {
                characters.next();
                in_line_comment = true;
                current.push(' ');
            }
            '/' if characters.peek() == Some(&'*') => {
                characters.next();
                in_block_comment = true;
                current.push(' ');
            }
            ';' => {
                if !current.trim().is_empty() {
                    statements.push(std::mem::take(&mut current));
                }
            }
            other => current.push(other.to_ascii_lowercase()),
        }
    }

    if in_single_quote || in_double_quote || in_block_comment {
        return Err(QueryExecutionError::SyntaxError);
    }
    if !current.trim().is_empty() {
        statements.push(current);
    }
    Ok(statements)
}

fn strip_literals_comments_and_check_semicolons(sql: &str) -> Result<String, QueryExecutionError> {
    let trimmed = sql.trim();
    let source = trimmed.strip_suffix(';').unwrap_or(trimmed);
    let mut normalized = String::with_capacity(source.len());
    let mut characters = source.chars().peekable();
    let mut in_single_quote = false;
    let mut in_double_quote = false;
    let mut in_line_comment = false;
    let mut in_block_comment = false;

    while let Some(character) = characters.next() {
        if in_line_comment {
            if character == '\n' {
                in_line_comment = false;
                normalized.push(' ');
            }
            continue;
        }
        if in_block_comment {
            if character == '*' && characters.peek() == Some(&'/') {
                characters.next();
                in_block_comment = false;
                normalized.push(' ');
            }
            continue;
        }
        if in_single_quote {
            if character == '\'' {
                if characters.peek() == Some(&'\'') {
                    characters.next();
                } else {
                    in_single_quote = false;
                }
            }
            normalized.push(' ');
            continue;
        }
        if in_double_quote {
            if character == '"' {
                in_double_quote = false;
            } else {
                normalized.push(character.to_ascii_lowercase());
            }
            continue;
        }

        match character {
            '\'' => {
                in_single_quote = true;
                normalized.push(' ');
            }
            '"' => {
                in_double_quote = true;
                normalized.push(' ');
            }
            '-' if characters.peek() == Some(&'-') => {
                characters.next();
                in_line_comment = true;
                normalized.push(' ');
            }
            '/' if characters.peek() == Some(&'*') => {
                characters.next();
                in_block_comment = true;
                normalized.push(' ');
            }
            ';' => return Err(QueryExecutionError::PolicyViolation),
            other => normalized.push(other.to_ascii_lowercase()),
        }
    }

    if in_single_quote || in_double_quote || in_block_comment {
        return Err(QueryExecutionError::SyntaxError);
    }

    Ok(normalized)
}

fn logical_type(postgres_type: &Type) -> Result<LogicalType, QueryExecutionError> {
    match *postgres_type {
        Type::INT2 | Type::INT4 => Ok(LogicalType::Integer),
        Type::INT8 => Ok(LogicalType::Bigint),
        Type::NUMERIC => Ok(LogicalType::Numeric),
        Type::FLOAT4 | Type::FLOAT8 => Ok(LogicalType::Float),
        Type::BOOL => Ok(LogicalType::Boolean),
        Type::TEXT | Type::VARCHAR | Type::CHAR | Type::NAME => Ok(LogicalType::Text),
        Type::BYTEA => Ok(LogicalType::Binary),
        Type::DATE => Ok(LogicalType::Date),
        Type::TIMESTAMP => Ok(LogicalType::Timestamp),
        Type::TIMESTAMPTZ => Ok(LogicalType::Timestamptz),
        Type::UUID => Ok(LogicalType::Uuid),
        Type::JSON | Type::JSONB => Ok(LogicalType::Json),
        _ => Err(QueryExecutionError::UnsupportedType(
            postgres_type.name().to_owned(),
        )),
    }
}

fn decode_row(row: &Row) -> Result<Vec<SqlValue>, QueryExecutionError> {
    row.columns()
        .iter()
        .enumerate()
        .map(|(index, column)| match *column.type_() {
            Type::BOOL => row
                .try_get::<_, Option<bool>>(index)
                .map(|value| value.map_or(SqlValue::Null, SqlValue::Bool))
                .map_err(|_| QueryExecutionError::RuntimeError),
            Type::INT2 => row
                .try_get::<_, Option<i16>>(index)
                .map(|value| value.map_or(SqlValue::Null, |value| SqlValue::Signed(value as i64)))
                .map_err(|_| QueryExecutionError::RuntimeError),
            Type::INT4 => row
                .try_get::<_, Option<i32>>(index)
                .map(|value| value.map_or(SqlValue::Null, |value| SqlValue::Signed(value as i64)))
                .map_err(|_| QueryExecutionError::RuntimeError),
            Type::INT8 => row
                .try_get::<_, Option<i64>>(index)
                .map(|value| value.map_or(SqlValue::Null, SqlValue::Signed))
                .map_err(|_| QueryExecutionError::RuntimeError),
            Type::NUMERIC => Err(QueryExecutionError::UnsupportedType("numeric".to_owned())),
            Type::FLOAT4 => row
                .try_get::<_, Option<f32>>(index)
                .map(|value| {
                    value.map_or(SqlValue::Null, |value| SqlValue::Float {
                        bits: value.to_bits() as u64,
                    })
                })
                .map_err(|_| QueryExecutionError::RuntimeError),
            Type::FLOAT8 => row
                .try_get::<_, Option<f64>>(index)
                .map(|value| {
                    value.map_or(SqlValue::Null, |value| SqlValue::Float {
                        bits: value.to_bits(),
                    })
                })
                .map_err(|_| QueryExecutionError::RuntimeError),
            Type::TEXT | Type::VARCHAR | Type::CHAR | Type::NAME => row
                .try_get::<_, Option<String>>(index)
                .map(|value| value.map_or(SqlValue::Null, SqlValue::Text))
                .map_err(|_| QueryExecutionError::RuntimeError),
            Type::BYTEA => row
                .try_get::<_, Option<Vec<u8>>>(index)
                .map(|value| value.map_or(SqlValue::Null, SqlValue::Bytes))
                .map_err(|_| QueryExecutionError::RuntimeError),
            Type::DATE => row
                .try_get::<_, Option<chrono::NaiveDate>>(index)
                .map(|value| {
                    value.map_or(SqlValue::Null, |value| SqlValue::Date(value.to_string()))
                })
                .map_err(|_| QueryExecutionError::RuntimeError),
            Type::TIMESTAMP => row
                .try_get::<_, Option<chrono::NaiveDateTime>>(index)
                .map(|value| {
                    value.map_or(SqlValue::Null, |value| {
                        SqlValue::Timestamp(value.to_string())
                    })
                })
                .map_err(|_| QueryExecutionError::RuntimeError),
            Type::TIMESTAMPTZ => row
                .try_get::<_, Option<chrono::DateTime<chrono::Utc>>>(index)
                .map(|value| {
                    value.map_or(SqlValue::Null, |value| {
                        SqlValue::Timestamptz(value.to_rfc3339())
                    })
                })
                .map_err(|_| QueryExecutionError::RuntimeError),
            Type::UUID => row
                .try_get::<_, Option<uuid::Uuid>>(index)
                .map(|value| {
                    value.map_or(SqlValue::Null, |value| SqlValue::Uuid(value.to_string()))
                })
                .map_err(|_| QueryExecutionError::RuntimeError),
            Type::JSON | Type::JSONB => row
                .try_get::<_, Option<serde_json::Value>>(index)
                .map_err(|_| QueryExecutionError::RuntimeError)
                .and_then(|value| {
                    value.map_or(Ok(SqlValue::Null), |value| {
                        serde_json::to_string(&value)
                            .map(SqlValue::Json)
                            .map_err(|_| QueryExecutionError::RuntimeError)
                    })
                })
                .map_err(|_| QueryExecutionError::RuntimeError),
            _ => Err(QueryExecutionError::UnsupportedType(
                column.type_().name().to_owned(),
            )),
        })
        .collect()
}

fn map_database_error(error: postgres::Error) -> QueryExecutionError {
    if let Some(database_error) = error.as_db_error() {
        match database_error.code().code() {
            "57014" => return QueryExecutionError::TimeLimitExceeded,
            "42601" | "42602" | "42603" | "42604" | "42609" | "42610" => {
                return QueryExecutionError::SyntaxError;
            }
            _ => {}
        }
    }
    QueryExecutionError::RuntimeError
}
