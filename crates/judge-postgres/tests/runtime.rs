use judge_core::{QueryLimits, SqlValue};
use judge_postgres::{ManagedPostgresRuntime, ManagedPostgresSession};
use std::fs;

#[test]
fn managed_runtime_starts_owned_socket_instance() {
    let root = std::env::temp_dir().join(format!(
        "sql-judge-runtime-{}-{}",
        std::process::id(),
        unique_suffix()
    ));
    let runtime = ManagedPostgresRuntime::start(&root, "runtime-test")
        .expect("start app-owned postgres runtime");
    let mut session = ManagedPostgresSession::connect(&runtime.connection_string(), "runtime-test")
        .expect("connect marked managed runtime");

    let result = session
        .execute(
            "SELECT current_database()",
            QueryLimits::new(2_000, 10, 64 * 1024),
        )
        .expect("query managed runtime");
    assert_eq!(result.rows.len(), 1);
    assert!(matches!(result.rows[0][0], SqlValue::Text(_)));

    drop(runtime);
    assert!(!root.join("postmaster.pid").exists());
    let _ = fs::remove_dir_all(root);
}

#[test]
fn managed_runtime_restarts_only_with_matching_instance_marker() {
    let root = std::env::temp_dir().join(format!(
        "sql-judge-runtime-restart-{}-{}",
        std::process::id(),
        unique_suffix()
    ));
    let runtime =
        ManagedPostgresRuntime::start(&root, "runtime-restart").expect("start initial runtime");
    drop(runtime);

    let restarted =
        ManagedPostgresRuntime::start(&root, "runtime-restart").expect("restart matching runtime");
    drop(restarted);

    let mismatched = ManagedPostgresRuntime::start(&root, "different-instance");
    assert!(matches!(
        mismatched,
        Err(judge_postgres::QueryExecutionError::OwnershipVerificationFailed)
    ));
    let _ = fs::remove_dir_all(root);
}

fn unique_suffix() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("system clock")
        .as_nanos()
}
