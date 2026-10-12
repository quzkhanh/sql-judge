use crate::{
    DestructiveAction, EnvironmentRecord, EnvironmentState, OwnershipError,
    authorize_destructive_action,
};
use std::collections::BTreeMap;
use thiserror::Error;

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum LifecycleError {
    #[error("environment already exists")]
    Duplicate,
    #[error("environment namespace is already assigned")]
    NamespaceConflict,
    #[error("environment namespace is invalid")]
    InvalidNamespace,
    #[error("environment was not found")]
    NotFound,
    #[error("replacement environment failed validation")]
    ReplacementValidationFailed,
    #[error("environment ownership check failed")]
    Ownership(#[from] OwnershipError),
}

#[derive(Debug, Default)]
pub struct EnvironmentLedger {
    records: BTreeMap<String, EnvironmentRecord>,
}

impl EnvironmentLedger {
    pub fn restore<I>(records: I) -> Result<Self, LifecycleError>
    where
        I: IntoIterator<Item = EnvironmentRecord>,
    {
        let mut ledger = Self::default();
        for record in records {
            ledger.create(record)?;
        }
        Ok(ledger)
    }

    pub fn create(&mut self, record: EnvironmentRecord) -> Result<(), LifecycleError> {
        validate_namespace(&record.namespace)?;
        if self.records.contains_key(&record.env_id) {
            return Err(LifecycleError::Duplicate);
        }
        if self
            .records
            .values()
            .any(|existing| existing.namespace == record.namespace)
        {
            return Err(LifecycleError::NamespaceConflict);
        }
        self.records.insert(record.env_id.clone(), record);
        Ok(())
    }

    pub fn clone_environment(
        &mut self,
        source_id: &str,
        mut replacement: EnvironmentRecord,
    ) -> Result<EnvironmentRecord, LifecycleError> {
        let source = self
            .records
            .get(source_id)
            .ok_or(LifecycleError::NotFound)?;
        if source.mode != crate::EnvironmentMode::Managed {
            return Err(LifecycleError::Ownership(
                OwnershipError::ExternalEnvironment,
            ));
        }
        replacement.pack_id = source.pack_id.clone();
        self.create(replacement.clone())?;
        Ok(replacement)
    }

    pub fn inspect(&self, env_id: &str) -> Result<&EnvironmentRecord, LifecycleError> {
        self.records.get(env_id).ok_or(LifecycleError::NotFound)
    }

    pub fn list(&self) -> Vec<&EnvironmentRecord> {
        self.records.values().collect()
    }

    pub fn rebuild<F>(
        &mut self,
        env_id: &str,
        mut replacement: EnvironmentRecord,
        action: DestructiveAction,
        validate: F,
    ) -> Result<(), LifecycleError>
    where
        F: FnOnce(&EnvironmentRecord) -> bool,
    {
        let current = self.records.get(env_id).ok_or(LifecycleError::NotFound)?;
        require_reset_action(action.clone())?;
        authorize_destructive_action(current, action)?;
        validate_replacement(current, &replacement)?;
        if !validate(&replacement) {
            return Err(LifecycleError::ReplacementValidationFailed);
        }
        validate_namespace(&replacement.namespace)?;
        if self
            .records
            .iter()
            .any(|(id, record)| id.as_str() != env_id && record.namespace == replacement.namespace)
        {
            return Err(LifecycleError::NamespaceConflict);
        }
        replacement.env_id = env_id.to_owned();
        replacement.state = EnvironmentState::Ready;
        self.records.insert(env_id.to_owned(), replacement);
        Ok(())
    }

    pub fn begin_rebuild(
        &mut self,
        env_id: &str,
        action: DestructiveAction,
    ) -> Result<(), LifecycleError> {
        let record = self.records.get(env_id).ok_or(LifecycleError::NotFound)?;
        require_reset_action(action.clone())?;
        authorize_destructive_action(record, action)?;
        self.records
            .get_mut(env_id)
            .expect("rebuild record still exists")
            .state = EnvironmentState::Rebuilding;
        Ok(())
    }

    pub fn finish_rebuild<F>(
        &mut self,
        env_id: &str,
        mut replacement: EnvironmentRecord,
        action: DestructiveAction,
        validate: F,
    ) -> Result<(), LifecycleError>
    where
        F: FnOnce(&EnvironmentRecord) -> bool,
    {
        let current = self.records.get(env_id).ok_or(LifecycleError::NotFound)?;
        authorize_lifecycle_completion(current, action, EnvironmentState::Rebuilding)?;
        let pack_id = current.pack_id.clone();
        validate_replacement(current, &replacement)?;
        if !validate(&replacement) {
            self.records
                .get_mut(env_id)
                .expect("rebuild record still exists")
                .state = EnvironmentState::Ready;
            return Err(LifecycleError::ReplacementValidationFailed);
        }
        let namespace_valid = validate_namespace(&replacement.namespace).is_ok();
        let namespace_conflicts = self
            .records
            .iter()
            .any(|(id, record)| id.as_str() != env_id && record.namespace == replacement.namespace);
        if !namespace_valid || namespace_conflicts {
            self.records
                .get_mut(env_id)
                .expect("rebuild record still exists")
                .state = EnvironmentState::Ready;
            return if !namespace_valid {
                Err(LifecycleError::InvalidNamespace)
            } else {
                Err(LifecycleError::NamespaceConflict)
            };
        }
        replacement.env_id = env_id.to_owned();
        replacement.pack_id = pack_id;
        replacement.state = EnvironmentState::Ready;
        self.records.insert(env_id.to_owned(), replacement);
        Ok(())
    }

    pub fn mark_running(&mut self, env_id: &str) -> Result<(), LifecycleError> {
        let record = self
            .records
            .get_mut(env_id)
            .ok_or(LifecycleError::NotFound)?;
        if record.state != EnvironmentState::Ready {
            return Err(LifecycleError::Ownership(
                if matches!(
                    record.state,
                    EnvironmentState::Running
                        | EnvironmentState::Rebuilding
                        | EnvironmentState::Deleting
                ) {
                    OwnershipError::Busy
                } else {
                    OwnershipError::InvalidState
                },
            ));
        }
        record.state = EnvironmentState::Running;
        Ok(())
    }

    pub fn mark_ready(&mut self, env_id: &str) -> Result<(), LifecycleError> {
        let record = self
            .records
            .get_mut(env_id)
            .ok_or(LifecycleError::NotFound)?;
        if record.state != EnvironmentState::Provisioning {
            return Err(LifecycleError::Ownership(OwnershipError::InvalidState));
        }
        record.state = EnvironmentState::Ready;
        Ok(())
    }

    pub fn mark_error(&mut self, env_id: &str) -> Result<(), LifecycleError> {
        let record = self
            .records
            .get_mut(env_id)
            .ok_or(LifecycleError::NotFound)?;
        if !matches!(
            record.state,
            EnvironmentState::Provisioning
                | EnvironmentState::Ready
                | EnvironmentState::Rebuilding
                | EnvironmentState::Deleting
        ) {
            return Err(LifecycleError::Ownership(OwnershipError::InvalidState));
        }
        record.state = EnvironmentState::Error;
        Ok(())
    }

    pub fn begin_delete(
        &mut self,
        env_id: &str,
        action: DestructiveAction,
    ) -> Result<(), LifecycleError> {
        let record = self.records.get(env_id).ok_or(LifecycleError::NotFound)?;
        require_delete_action(action.clone())?;
        authorize_destructive_action(record, action)?;
        self.records
            .get_mut(env_id)
            .expect("delete record still exists")
            .state = EnvironmentState::Deleting;
        Ok(())
    }

    pub fn finish_delete(
        &mut self,
        env_id: &str,
        action: DestructiveAction,
    ) -> Result<EnvironmentRecord, LifecycleError> {
        let record = self.records.get(env_id).ok_or(LifecycleError::NotFound)?;
        authorize_lifecycle_completion(record, action, EnvironmentState::Deleting)?;
        self.records.remove(env_id).ok_or(LifecycleError::NotFound)
    }

    pub fn delete(
        &mut self,
        env_id: &str,
        action: DestructiveAction,
    ) -> Result<EnvironmentRecord, LifecycleError> {
        let record = self.records.get(env_id).ok_or(LifecycleError::NotFound)?;
        require_delete_action(action.clone())?;
        authorize_destructive_action(record, action)?;
        self.records.remove(env_id).ok_or(LifecycleError::NotFound)
    }
}

fn authorize_lifecycle_completion(
    record: &EnvironmentRecord,
    action: DestructiveAction,
    expected_state: EnvironmentState,
) -> Result<(), LifecycleError> {
    if record.state != expected_state {
        return Err(LifecycleError::Ownership(OwnershipError::InvalidState));
    }
    if record.mode != crate::EnvironmentMode::Managed {
        return Err(LifecycleError::Ownership(
            OwnershipError::ExternalEnvironment,
        ));
    }
    let (env_id, instance_id, nonce, confirmed) = match (expected_state, action) {
        (
            EnvironmentState::Rebuilding,
            DestructiveAction::Reset {
                env_id,
                instance_id,
                ownership_nonce,
                confirmed,
            },
        )
        | (
            EnvironmentState::Deleting,
            DestructiveAction::Delete {
                env_id,
                instance_id,
                ownership_nonce,
                confirmed,
            },
        ) => (env_id, instance_id, ownership_nonce, confirmed),
        _ => {
            return Err(LifecycleError::Ownership(OwnershipError::InvalidState));
        }
    };
    if env_id != record.env_id {
        return Err(LifecycleError::Ownership(
            OwnershipError::EnvironmentMismatch,
        ));
    }
    if instance_id != record.instance_id {
        return Err(LifecycleError::Ownership(OwnershipError::InstanceMismatch));
    }
    if nonce != record.ownership_nonce {
        return Err(LifecycleError::Ownership(OwnershipError::OwnershipMismatch));
    }
    if !confirmed {
        return Err(LifecycleError::Ownership(
            OwnershipError::ConfirmationRequired,
        ));
    }
    Ok(())
}

fn validate_replacement(
    current: &EnvironmentRecord,
    replacement: &EnvironmentRecord,
) -> Result<(), LifecycleError> {
    if replacement.mode != crate::EnvironmentMode::Managed {
        return Err(LifecycleError::Ownership(
            OwnershipError::ExternalEnvironment,
        ));
    }
    if replacement.instance_id != current.instance_id {
        return Err(LifecycleError::Ownership(OwnershipError::InstanceMismatch));
    }
    if replacement.ownership_nonce != current.ownership_nonce {
        return Err(LifecycleError::Ownership(OwnershipError::OwnershipMismatch));
    }
    Ok(())
}

fn require_reset_action(action: DestructiveAction) -> Result<(), LifecycleError> {
    if matches!(action, DestructiveAction::Reset { .. }) {
        Ok(())
    } else {
        Err(LifecycleError::Ownership(OwnershipError::InvalidState))
    }
}

fn require_delete_action(action: DestructiveAction) -> Result<(), LifecycleError> {
    if matches!(action, DestructiveAction::Delete { .. }) {
        Ok(())
    } else {
        Err(LifecycleError::Ownership(OwnershipError::InvalidState))
    }
}

fn validate_namespace(namespace: &str) -> Result<(), LifecycleError> {
    let bytes = namespace.as_bytes();
    if bytes.is_empty()
        || bytes.len() > 63
        || !namespace.starts_with("sj_")
        || !bytes
            .iter()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || *byte == b'_')
    {
        return Err(LifecycleError::InvalidNamespace);
    }
    Ok(())
}
