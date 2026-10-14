use judge_sqlserver::{
    SqlServerConfig, SqlServerError, SqlServerExplorer, SqlServerTlsMode, validate_read_only_tsql,
};

fn config(tls_mode: SqlServerTlsMode) -> SqlServerConfig {
    SqlServerConfig {
        host: "sql.example.test".to_owned(),
        port: 1433,
        database: "practice".to_owned(),
        username: "reader".to_owned(),
        password: "secret-canary".to_owned(),
        tls_mode,
    }
}

#[test]
fn policy_rejects_tsql_writes_execution_and_cross_database_access() {
    for sql in [
        "INSERT INTO dbo.users VALUES (1)",
        "UPDATE dbo.users SET name = 'changed'",
        "DELETE FROM dbo.users",
        "DROP TABLE dbo.users",
        "EXEC dbo.usp_mutate",
        "EXEC xp_cmdshell 'whoami'",
        "SELECT * FROM OPENQUERY([remote], 'SELECT 1')",
        "SELECT * FROM OPENROWSET('SQLNCLI', 'Server=other', 'SELECT 1')",
        "WAITFOR DELAY '00:00:01'",
        "SELECT * INTO dbo.copy_users FROM dbo.users",
        "SELECT * FROM other_database.dbo.users",
        "SELECT * FROM other_database..users",
        "SELECT * FROM [other_database]..[users]",
        "SELECT * FROM other_database.sales.orders",
        "SELECT * FROM dbo.users JOIN other_database.dbo.roles ON 1 = 1",
        "SELECT * FROM dbo.users JOIN other_database.reporting.orders ON 1 = 1",
        "SELECT * FROM dbo.users, other_database.sales.orders",
        "SELECT 1; SELECT 2",
        "SELECT 'unterminated",
    ] {
        assert_eq!(
            validate_read_only_tsql(sql),
            Err(SqlServerError::QueryRejected),
            "unexpected policy result for {sql}"
        );
    }
}

#[test]
fn policy_accepts_bounded_select_and_cte() {
    assert_eq!(
        validate_read_only_tsql("SELECT TOP (10) id FROM dbo.users"),
        Ok(())
    );
    assert_eq!(validate_read_only_tsql("SELECT id FROM dbo.users"), Ok(()));
    assert_eq!(
        validate_read_only_tsql("SELECT id FROM [dbo].[users]"),
        Ok(())
    );
    assert_eq!(
        validate_read_only_tsql("WITH rows AS (SELECT id FROM dbo.users) SELECT id FROM rows"),
        Ok(())
    );
    assert_eq!(
        validate_read_only_tsql("SELECT 'insert and DROP are literal values'"),
        Ok(())
    );
}

#[test]
fn tls_disable_and_trust_certificate_are_rejected() {
    assert!(matches!(
        SqlServerExplorer::connect(config(SqlServerTlsMode::Disable)),
        Err(SqlServerError::TlsPolicyRejected)
    ));
    assert!(matches!(
        SqlServerExplorer::connect(config(SqlServerTlsMode::TrustServerCertificate)),
        Err(SqlServerError::TlsPolicyRejected)
    ));
    assert!(matches!(
        SqlServerExplorer::connect(config(SqlServerTlsMode::VerifyCa)),
        Err(SqlServerError::TlsPolicyRejected)
    ));
}
