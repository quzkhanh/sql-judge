use app_storage::{
    EnvironmentStore, HistoryStore, RecoveryOperation, RecoveryRecord, SubmissionRecord, TestRecord,
};
use environment_manager::{EnvironmentMode, EnvironmentRecord, EnvironmentState};

fn record(id: &str) -> SubmissionRecord {
    SubmissionRecord {
        submission_id: id.to_owned(),
        pack_id: "hr-sql-101".to_owned(),
        pack_version: "1.0.0".to_owned(),
        pack_hash: "sha256:test".to_owned(),
        engine_version: "postgresql-18".to_owned(),
        policy_version: "policy-v1".to_owned(),
        verdict: "Accepted".to_owned(),
        created_at: "2026-10-12T00:00:00Z".to_owned(),
    }
}

fn test_record(id: &str) -> TestRecord {
    TestRecord {
        test_id: id.to_owned(),
        verdict: "Accepted".to_owned(),
        elapsed_ms: 12,
    }
}

fn environment(id: &str, state: EnvironmentState) -> EnvironmentRecord {
    EnvironmentRecord {
        env_id: id.to_owned(),
        instance_id: format!("instance-{id}"),
        pack_id: "hr-sql-101".to_owned(),
        namespace: format!("sj_{id}").replace('-', "_"),
        ownership_nonce: format!("nonce-{id}"),
        mode: EnvironmentMode::Managed,
        state,
    }
}

#[test]
fn history_survives_store_reopen_without_credentials() {
    let path =
        std::env::temp_dir().join(format!("sql-judge-history-{}.sqlite", std::process::id()));
    let _ = std::fs::remove_file(&path);

    {
        let mut store = HistoryStore::open(&path).expect("open history");
        store
            .save_submission(&record("submission-1"), &[test_record("basic")])
            .expect("save submission");
    }

    let store = HistoryStore::open(&path).expect("reopen history");
    let submissions = store.list_submissions().expect("list submissions");
    assert_eq!(submissions.len(), 1);
    assert_eq!(submissions[0].pack_hash, "sha256:test");
    assert!(!submissions[0].pack_hash.contains("password"));
    let _ = std::fs::remove_file(path);
}

#[test]
fn runtime_instance_id_survives_store_reopen() {
    let path = std::env::temp_dir().join(format!(
        "sql-judge-runtime-id-{}.sqlite",
        std::process::id()
    ));
    let _ = std::fs::remove_file(&path);
    let first = {
        let mut store = EnvironmentStore::open(&path).expect("open store");
        store
            .get_or_create_runtime_instance_id()
            .expect("create runtime identity")
    };
    let second = {
        let mut store = EnvironmentStore::open(&path).expect("reopen store");
        store
            .get_or_create_runtime_instance_id()
            .expect("restore runtime identity")
    };

    assert_eq!(first, second);
    assert!(first.starts_with("runtime-"));
    let _ = std::fs::remove_file(path);
}

#[test]
fn environment_store_can_remove_deleted_environment() {
    let path = std::env::temp_dir().join(format!(
        "sql-judge-environment-delete-{}.sqlite",
        std::process::id()
    ));
    let _ = std::fs::remove_file(&path);
    let mut store = EnvironmentStore::open(&path).expect("open store");
    store
        .save_environment(&environment("env-delete", EnvironmentState::Deleting))
        .expect("save deleting environment");

    store
        .delete_environment("env-delete")
        .expect("delete environment");
    assert!(
        store
            .get_environment("env-delete")
            .expect("read deleted")
            .is_none()
    );
    let _ = std::fs::remove_file(path);
}

#[test]
fn failed_submission_write_rolls_back_all_test_rows() {
    let path = std::env::temp_dir().join(format!(
        "sql-judge-history-rollback-{}.sqlite",
        std::process::id()
    ));
    let _ = std::fs::remove_file(&path);
    let mut store = HistoryStore::open(&path).expect("open history");
    store
        .save_submission(&record("submission-1"), &[test_record("basic")])
        .expect("save initial submission");

    let error = store.save_submission(
        &record("submission-1"),
        &[test_record("should-not-persist")],
    );
    assert!(error.is_err());
    assert_eq!(store.test_count("submission-1").expect("count tests"), 1);
    let _ = std::fs::remove_file(path);
}

#[test]
fn environment_ledger_survives_reopen_and_updates_state_without_credentials() {
    let path = std::env::temp_dir().join(format!(
        "sql-judge-environments-{}.sqlite",
        std::process::id()
    ));
    let _ = std::fs::remove_file(&path);

    {
        let mut store = EnvironmentStore::open(&path).expect("open environments");
        store
            .save_environment(&environment("env-a", EnvironmentState::Provisioning))
            .expect("save provisioning environment");
        store
            .save_environment(&environment("env-a", EnvironmentState::Ready))
            .expect("update environment state");
    }

    let store = EnvironmentStore::open(&path).expect("reopen environments");
    let restored = store
        .get_environment("env-a")
        .expect("load environment")
        .expect("environment exists");
    assert_eq!(restored.state, EnvironmentState::Ready);
    assert_eq!(restored.namespace, "sj_env_a");
    assert!(!restored.ownership_nonce.contains("password"));
    let _ = std::fs::remove_file(path);
}

#[test]
fn environment_ledger_lists_records_for_restart_reconciliation() {
    let path = std::env::temp_dir().join(format!(
        "sql-judge-environments-list-{}.sqlite",
        std::process::id()
    ));
    let _ = std::fs::remove_file(&path);

    let mut store = EnvironmentStore::open(&path).expect("open environments");
    store
        .save_environment(&environment("env-a", EnvironmentState::Ready))
        .expect("save first");
    store
        .save_environment(&environment("env-b", EnvironmentState::NeedsRecovery))
        .expect("save second");

    let records = store.list_environments().expect("list environments");
    assert_eq!(records.len(), 2);
    assert_eq!(records[0].env_id, "env-a");
    assert_eq!(records[1].state, EnvironmentState::NeedsRecovery);
    let _ = std::fs::remove_file(path);
}

#[test]
fn environment_store_rejects_unowned_namespace_names() {
    let path = std::env::temp_dir().join(format!(
        "sql-judge-environments-invalid-{}.sqlite",
        std::process::id()
    ));
    let _ = std::fs::remove_file(&path);
    let mut store = EnvironmentStore::open(&path).expect("open environments");
    let mut record = environment("env-a", EnvironmentState::Ready);
    record.namespace = "public".to_owned();

    assert!(matches!(
        store.save_environment(&record),
        Err(app_storage::StorageError::InvalidEnvironment("namespace"))
    ));
    assert!(
        store
            .get_environment("env-a")
            .expect("query environments")
            .is_none()
    );
    let _ = std::fs::remove_file(path);
}

#[test]
fn recovery_journal_survives_reopen_and_can_be_cleared() {
    let path =
        std::env::temp_dir().join(format!("sql-judge-recovery-{}.sqlite", std::process::id()));
    let _ = std::fs::remove_file(&path);
    let recovery = RecoveryRecord {
        env_id: "env-a".to_owned(),
        operation: RecoveryOperation::Rebuild,
        instance_id: "instance-env-a".to_owned(),
        ownership_nonce: "nonce-env-a".to_owned(),
        namespace: "sj_env_a".to_owned(),
        replacement_namespace: Some("sj_env_a_next".to_owned()),
    };

    {
        let mut store = EnvironmentStore::open(&path).expect("open recovery store");
        store
            .save_recovery(&recovery)
            .expect("save recovery record");
    }

    let mut store = EnvironmentStore::open(&path).expect("reopen recovery store");
    let records = store.list_recoveries().expect("list recovery records");
    assert_eq!(records, vec![recovery]);
    store
        .clear_recovery("env-a")
        .expect("clear recovery record");
    assert!(
        store
            .list_recoveries()
            .expect("list cleared records")
            .is_empty()
    );
    let _ = std::fs::remove_file(path);
}
