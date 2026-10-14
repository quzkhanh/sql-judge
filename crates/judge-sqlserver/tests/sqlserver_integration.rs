#![cfg(feature = "sqlserver-integration")]

use judge_core::{LogicalType, QueryLimits, SqlValue};
use judge_sqlserver::{SqlServerConfig, SqlServerError, SqlServerExplorer, SqlServerTlsMode};

fn fixture_config() -> SqlServerConfig {
    SqlServerConfig {
        host: std::env::var("SQLSERVER_TEST_HOST").expect("SQLSERVER_TEST_HOST is required"),
        port: std::env::var("SQLSERVER_TEST_PORT")
            .expect("SQLSERVER_TEST_PORT is required")
            .parse()
            .expect("SQLSERVER_TEST_PORT must be numeric"),
        database: std::env::var("SQLSERVER_TEST_DATABASE")
            .expect("SQLSERVER_TEST_DATABASE is required"),
        username: std::env::var("SQLSERVER_TEST_USERNAME")
            .expect("SQLSERVER_TEST_USERNAME is required"),
        password: std::env::var("SQLSERVER_TEST_PASSWORD")
            .expect("SQLSERVER_TEST_PASSWORD is required"),
        tls_mode: SqlServerTlsMode::VerifyFull,
    }
}

#[test]
#[ignore = "requires the external SQL Server fixture configured by test_sqlserver_harness.sh"]
fn disposable_fixture_supports_read_only_explorer_operations() {
    let explorer = SqlServerExplorer::connect(fixture_config()).expect("connect fixture");
    let connection = explorer.test_connection().expect("test connection");
    assert_eq!(connection.database, "sqljudge_fixture");
    assert!(connection.read_only);

    let metadata = explorer
        .list_metadata(QueryLimits::new(2_000, 100, 128 * 1024))
        .expect("list metadata");
    assert!(
        metadata
            .tables
            .iter()
            .any(|table| table.schema == "dbo" && table.name == "sample")
    );

    let result = explorer
        .execute_select(
            "SELECT TOP (2) id, label, amount, active, item_id, payload FROM dbo.sample ORDER BY id",
            QueryLimits::new(2_000, 2, 128 * 1024),
        )
        .expect("execute bounded select");
    assert_eq!(result.columns[0].logical_type, LogicalType::Integer);
    assert_eq!(result.columns[2].logical_type, LogicalType::Numeric);
    assert_eq!(result.rows.len(), 2);
    assert!(matches!(result.rows[0][0], SqlValue::Signed(1)));
    assert!(matches!(
        result.rows[1][1],
        SqlValue::Null | SqlValue::Text(_)
    ));
    assert!(matches!(result.rows[0][2], SqlValue::Decimal(_)));
    assert!(matches!(result.rows[0][3], SqlValue::Bool(true)));
    assert!(matches!(result.rows[0][4], SqlValue::Uuid(_)));
    assert!(matches!(result.rows[0][5], SqlValue::Bytes(_)));
}

#[test]
#[ignore = "requires the external SQL Server fixture configured by test_sqlserver_harness.sh"]
fn adapter_rejects_mutating_statements_before_server_execution() {
    let explorer = SqlServerExplorer::connect(fixture_config()).expect("connect fixture");
    for sql in [
        "INSERT INTO dbo.sample (id) VALUES (99)",
        "DROP TABLE dbo.sample",
    ] {
        assert_eq!(
            explorer.execute_select(sql, QueryLimits::new(2_000, 10, 16 * 1024)),
            Err(SqlServerError::QueryRejected)
        );
    }
}

#[test]
#[ignore = "requires the external SQL Server fixture configured by test_sqlserver_harness.sh"]
fn adapter_rejects_a_same_database_view_with_an_indirect_cross_database_dependency() {
    let explorer = SqlServerExplorer::connect(fixture_config()).expect("connect fixture");
    assert_eq!(
        explorer.execute_select(
            "SELECT id FROM dbo.indirect_other",
            QueryLimits::new(2_000, 10, 16 * 1024),
        ),
        Err(SqlServerError::QueryRejected)
    );
}
