use judge_core::{QueryLimits, SqlValue};
use judge_postgres::{ManagedPostgresRuntime, ManagedPostgresSession};
use std::fs;
use std::process::{Command, Stdio};

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
    let recovered = ManagedPostgresRuntime::start(&root, "runtime-restart")
        .expect("mismatched ownership check must not stop the owned process");
    drop(recovered);
    let _ = fs::remove_dir_all(root);
}

#[test]
fn managed_runtime_adopts_owned_process_left_after_app_crash() {
    let root = std::env::temp_dir().join(format!(
        "sql-judge-runtime-adopt-{}-{}",
        std::process::id(),
        unique_suffix()
    ));
    let runtime =
        ManagedPostgresRuntime::start(&root, "runtime-adopt").expect("start initial runtime");
    std::mem::forget(runtime);

    let mismatched = ManagedPostgresRuntime::adopt(&root, "different-instance");
    assert!(matches!(
        mismatched,
        Err(judge_postgres::QueryExecutionError::OwnershipVerificationFailed)
    ));

    let adopted = ManagedPostgresRuntime::adopt(&root, "runtime-adopt");
    match adopted {
        Ok(runtime) => {
            drop(runtime);
            let _ = fs::remove_dir_all(root);
        }
        Err(error) => {
            let _ = Command::new("pg_ctl")
                .args([
                    "-D",
                    root.to_str().expect("runtime root"),
                    "-w",
                    "stop",
                    "-m",
                    "immediate",
                ])
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status();
            let _ = fs::remove_dir_all(&root);
            panic!("expected adoption of running managed runtime: {error:?}");
        }
    }
}

#[test]
fn managed_runtime_startup_adoption_does_not_start_a_stopped_instance() {
    let root = std::env::temp_dir().join(format!(
        "sql-judge-runtime-startup-adopt-{}-{}",
        std::process::id(),
        unique_suffix()
    ));
    let runtime = ManagedPostgresRuntime::start(&root, "runtime-startup-adopt")
        .expect("start initial runtime");
    drop(runtime);

    let adopted = ManagedPostgresRuntime::adopt(&root, "runtime-startup-adopt");

    assert!(matches!(
        adopted,
        Err(judge_postgres::QueryExecutionError::ConnectionError)
    ));
    assert!(!root.join("postmaster.pid").exists());
    let _ = fs::remove_dir_all(root);
}

fn unique_suffix() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("system clock")
        .as_nanos()
}
