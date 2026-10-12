use judge_postgres::{
    PostgresSession, QueryExecutionError, validate_read_only_sql, validate_setup_sql,
};

#[test]
fn policy_rejects_multiple_statements_and_writes() {
    for sql in [
        "SELECT 1; SELECT 2",
        "INSERT INTO users VALUES (1)",
        "WITH changed AS (DELETE FROM users RETURNING id) SELECT * FROM changed",
        "SELECT pg_sleep(1)",
        "SELECT \"pg_catalog\".\"pg_sleep\"(1)",
    ] {
        assert_eq!(
            validate_read_only_sql(sql),
            Err(QueryExecutionError::PolicyViolation)
        );
    }
}

#[test]
fn policy_accepts_select_and_read_only_with_queries() {
    assert_eq!(validate_read_only_sql("SELECT 1"), Ok(()));
    assert_eq!(validate_read_only_sql("SELECT 1;"), Ok(()));
    assert_eq!(
        validate_read_only_sql("WITH rows AS (SELECT 1 AS value) SELECT value FROM rows"),
        Ok(())
    );
}

#[test]
fn setup_policy_rejects_privileged_or_mutating_admin_sql() {
    for sql in [
        "CREATE ROLE attacker LOGIN PASSWORD 'secret'",
        "DROP TABLE users",
        "CREATE TABLE public.users (id integer)",
        "GRANT ALL ON SCHEMA public TO attacker",
        "DO $$ BEGIN PERFORM pg_sleep(1); END $$",
    ] {
        assert_eq!(
            validate_setup_sql(sql),
            Err(QueryExecutionError::PolicyViolation)
        );
    }
    assert_eq!(
        validate_setup_sql(
            "CREATE TABLE employees (id integer); INSERT INTO employees VALUES (1);"
        ),
        Ok(())
    );
}

#[test]
fn managed_session_rejects_non_loopback_connection_targets() {
    assert!(matches!(
        PostgresSession::connect("host=database.example user=runner dbname=practice"),
        Err(QueryExecutionError::PolicyViolation)
    ));
}
