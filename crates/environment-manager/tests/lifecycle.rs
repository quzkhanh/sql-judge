use environment_manager::{
    DestructiveAction, EnvironmentLedger, EnvironmentMode, EnvironmentRecord, EnvironmentState,
    LifecycleError,
};

fn record(env_id: &str, namespace: &str) -> EnvironmentRecord {
    EnvironmentRecord {
        env_id: env_id.to_owned(),
        instance_id: format!("instance-{env_id}"),
        pack_id: "hr-sql-101".to_owned(),
        namespace: namespace.to_owned(),
        ownership_nonce: format!("nonce-{env_id}"),
        mode: EnvironmentMode::Managed,
        state: EnvironmentState::Ready,
    }
}

fn reset_action(env_id: &str) -> DestructiveAction {
    DestructiveAction::Reset {
        env_id: env_id.to_owned(),
        instance_id: format!("instance-{env_id}"),
        ownership_nonce: format!("nonce-{env_id}"),
        confirmed: true,
    }
}

#[test]
fn create_and_clone_keep_same_pack_namespaces_isolated() {
    let mut ledger = EnvironmentLedger::default();
    ledger
        .create(record("env-a", "sj_a"))
        .expect("create first");
    let cloned = ledger
        .clone_environment("env-a", record("env-b", "sj_b"))
        .expect("clone second");

    assert_eq!(cloned.pack_id, "hr-sql-101");
    assert_ne!(
        ledger.inspect("env-a").expect("first").namespace,
        cloned.namespace
    );
    assert_eq!(ledger.list().len(), 2);
}

#[test]
fn clone_rejects_external_source_environment() {
    let mut ledger = EnvironmentLedger::default();
    let mut external = record("external-a", "sj_external");
    external.mode = EnvironmentMode::ExternalExplorer;
    ledger.create(external).expect("create external record");

    assert!(matches!(
        ledger.clone_environment("external-a", record("env-b", "sj_b")),
        Err(LifecycleError::Ownership(_))
    ));
}

#[test]
fn failed_rebuild_preserves_old_record_and_delete_requires_ownership() {
    let mut ledger = EnvironmentLedger::default();
    ledger.create(record("env-a", "sj_a")).expect("create");
    let failed = ledger.rebuild(
        "env-a",
        record("env-a", "sj_new"),
        reset_action("env-a"),
        |_| false,
    );
    assert_eq!(failed, Err(LifecycleError::ReplacementValidationFailed));
    assert_eq!(ledger.inspect("env-a").expect("old").namespace, "sj_a");

    let mut busy = record("env-a", "sj_a");
    busy.state = EnvironmentState::Running;
    ledger.create(busy).expect_err("duplicate busy record");
    ledger.mark_running("env-a").expect("mark busy");
    assert!(matches!(
        ledger.delete(
            "env-a",
            DestructiveAction::Delete {
                env_id: "env-a".to_owned(),
                instance_id: "instance-env-a".to_owned(),
                ownership_nonce: "nonce-env-a".to_owned(),
                confirmed: true,
            }
        ),
        Err(LifecycleError::Ownership(_))
    ));
}

#[test]
fn create_rejects_unsafe_or_reused_namespaces() {
    let mut ledger = EnvironmentLedger::default();
    ledger.create(record("env-a", "sj_a")).expect("create");

    assert_eq!(
        ledger.create(record("env-b", "sj_a")),
        Err(LifecycleError::NamespaceConflict)
    );
    assert_eq!(
        ledger.create(record("env-c", "sj-a")),
        Err(LifecycleError::InvalidNamespace)
    );
    assert_eq!(
        ledger.create(record("env-d", "public")),
        Err(LifecycleError::InvalidNamespace)
    );
}

#[test]
fn rebuild_rejects_namespace_collision_without_replacing_old_record() {
    let mut ledger = EnvironmentLedger::default();
    ledger
        .create(record("env-a", "sj_a"))
        .expect("create first");
    ledger
        .create(record("env-b", "sj_b"))
        .expect("create second");

    let replacement = record("env-a", "sj_b");
    assert_eq!(
        ledger.rebuild("env-a", replacement, reset_action("env-a"), |_| true),
        Err(LifecycleError::NamespaceConflict)
    );
    assert_eq!(
        ledger.inspect("env-a").expect("old record").namespace,
        "sj_a"
    );
}

#[test]
fn rebuild_lock_blocks_delete_until_replacement_is_ready() {
    let mut ledger = EnvironmentLedger::default();
    ledger.create(record("env-a", "sj_a")).expect("create");

    ledger
        .begin_rebuild("env-a", reset_action("env-a"))
        .expect("begin rebuild");
    assert_eq!(
        ledger.inspect("env-a").expect("rebuilding").state,
        EnvironmentState::Rebuilding
    );
    assert!(matches!(
        ledger.delete(
            "env-a",
            DestructiveAction::Delete {
                env_id: "env-a".to_owned(),
                instance_id: "instance-env-a".to_owned(),
                ownership_nonce: "nonce-env-a".to_owned(),
                confirmed: true,
            }
        ),
        Err(LifecycleError::Ownership(_))
    ));
    assert!(matches!(
        ledger.finish_rebuild(
            "env-a",
            record("env-a", "sj_rebuilt"),
            DestructiveAction::Delete {
                env_id: "env-a".to_owned(),
                instance_id: "instance-env-a".to_owned(),
                ownership_nonce: "nonce-env-a".to_owned(),
                confirmed: true,
            },
            |_| true,
        ),
        Err(LifecycleError::Ownership(_))
    ));

    ledger
        .finish_rebuild(
            "env-a",
            record("env-a", "sj_rebuilt"),
            reset_action("env-a"),
            |_| true,
        )
        .expect("finish rebuild");
    assert_eq!(
        ledger.inspect("env-a").expect("ready").state,
        EnvironmentState::Ready
    );
    assert_eq!(
        ledger.inspect("env-a").expect("ready").namespace,
        "sj_rebuilt"
    );
}

#[test]
fn delete_lock_requires_authorization_before_record_removal() {
    let mut ledger = EnvironmentLedger::default();
    ledger.create(record("env-a", "sj_a")).expect("create");
    let action = DestructiveAction::Delete {
        env_id: "env-a".to_owned(),
        instance_id: "instance-env-a".to_owned(),
        ownership_nonce: "nonce-env-a".to_owned(),
        confirmed: true,
    };

    ledger
        .begin_delete("env-a", action.clone())
        .expect("begin delete");
    assert_eq!(
        ledger.inspect("env-a").expect("deleting").state,
        EnvironmentState::Deleting
    );
    assert!(matches!(
        ledger.begin_delete("env-a", action.clone()),
        Err(LifecycleError::Ownership(_))
    ));
    let forged = DestructiveAction::Delete {
        env_id: "env-a".to_owned(),
        instance_id: "forged-instance".to_owned(),
        ownership_nonce: "nonce-env-a".to_owned(),
        confirmed: true,
    };
    assert!(matches!(
        ledger.finish_delete("env-a", forged),
        Err(LifecycleError::Ownership(_))
    ));
    assert!(matches!(
        ledger.finish_delete("env-a", reset_action("env-a")),
        Err(LifecycleError::Ownership(_))
    ));
    ledger
        .finish_delete("env-a", action)
        .expect("finish delete");
    assert_eq!(ledger.inspect("env-a"), Err(LifecycleError::NotFound));
}

#[test]
fn direct_delete_rejects_a_reset_action() {
    let mut ledger = EnvironmentLedger::default();
    ledger.create(record("env-a", "sj_a")).expect("create");

    assert!(matches!(
        ledger.delete("env-a", reset_action("env-a")),
        Err(LifecycleError::Ownership(_))
    ));
    assert!(ledger.inspect("env-a").is_ok());
}

#[test]
fn restore_rebuilds_ledger_and_rejects_conflicting_persisted_records() {
    let restored = EnvironmentLedger::restore([record("env-a", "sj_a"), record("env-b", "sj_b")])
        .expect("restore ledger");
    assert_eq!(restored.list().len(), 2);
    assert_eq!(
        restored
            .inspect("env-b")
            .expect("restored record")
            .namespace,
        "sj_b"
    );

    assert!(matches!(
        EnvironmentLedger::restore([record("env-a", "sj_a"), record("env-b", "sj_a")]),
        Err(LifecycleError::NamespaceConflict)
    ));
}

#[test]
fn recovery_state_cannot_start_a_running_job() {
    let mut ledger = EnvironmentLedger::default();
    let mut record = record("env-a", "sj_a");
    record.state = EnvironmentState::NeedsRecovery;
    ledger.create(record).expect("create recovery record");

    assert!(matches!(
        ledger.mark_running("env-a"),
        Err(LifecycleError::Ownership(_))
    ));
    assert_eq!(
        ledger.inspect("env-a").expect("recovery record").state,
        EnvironmentState::NeedsRecovery
    );
}

#[test]
fn provisioning_environment_can_be_marked_ready_only_after_setup() {
    let mut ledger = EnvironmentLedger::default();
    let mut pending = record("env-a", "sj_a");
    pending.state = EnvironmentState::Provisioning;
    ledger.create(pending).expect("create pending environment");

    ledger.mark_ready("env-a").expect("mark setup complete");
    assert_eq!(
        ledger.inspect("env-a").expect("ready environment").state,
        EnvironmentState::Ready
    );
}

#[test]
fn failed_lifecycle_cleanup_marks_environment_error() {
    let mut ledger = EnvironmentLedger::default();
    let mut record = record("env-a", "sj_a");
    record.state = EnvironmentState::Rebuilding;
    ledger
        .create(record)
        .expect("create rebuilding environment");

    ledger.mark_error("env-a").expect("mark lifecycle error");
    assert_eq!(
        ledger.inspect("env-a").expect("error environment").state,
        EnvironmentState::Error
    );
}

#[test]
fn recovery_can_restore_rebuild_and_remove_abandoned_operations() {
    let mut rebuilding = EnvironmentLedger::default();
    let mut rebuild_record = record("env-rebuild", "sj_rebuild");
    rebuild_record.state = EnvironmentState::NeedsRecovery;
    rebuilding
        .create(rebuild_record)
        .expect("create rebuild recovery");
    rebuilding
        .recover_rebuild("env-rebuild")
        .expect("restore rebuild");
    assert_eq!(
        rebuilding.inspect("env-rebuild").expect("restored").state,
        EnvironmentState::Ready
    );

    let mut deleting = EnvironmentLedger::default();
    let mut record = record("env-delete", "sj_delete");
    record.state = EnvironmentState::NeedsRecovery;
    deleting.create(record).expect("create delete recovery");
    deleting
        .remove_recovered("env-delete")
        .expect("remove abandoned operation");
    assert_eq!(
        deleting.inspect("env-delete"),
        Err(LifecycleError::NotFound)
    );
}
