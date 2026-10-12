use thiserror::Error;

mod ledger;

pub use ledger::{EnvironmentLedger, LifecycleError};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnvironmentMode {
    Managed,
    ExternalExplorer,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnvironmentState {
    Provisioning,
    Ready,
    Running,
    Rebuilding,
    Deleting,
    NeedsRecovery,
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnvironmentRecord {
    pub env_id: String,
    pub instance_id: String,
    pub pack_id: String,
    pub namespace: String,
    pub ownership_nonce: String,
    pub mode: EnvironmentMode,
    pub state: EnvironmentState,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DestructiveAction {
    Reset {
        env_id: String,
        instance_id: String,
        ownership_nonce: String,
        confirmed: bool,
    },
    Delete {
        env_id: String,
        instance_id: String,
        ownership_nonce: String,
        confirmed: bool,
    },
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum OwnershipError {
    #[error("destructive actions are only allowed for managed environments")]
    ExternalEnvironment,
    #[error("environment identity does not match the durable marker")]
    EnvironmentMismatch,
    #[error("database instance identity does not match the durable marker")]
    InstanceMismatch,
    #[error("ownership marker does not match")]
    OwnershipMismatch,
    #[error("explicit destructive confirmation is required")]
    ConfirmationRequired,
    #[error("environment is busy and cannot be destroyed")]
    Busy,
    #[error("environment is not in a destructible state")]
    InvalidState,
}

pub fn authorize_destructive_action(
    record: &EnvironmentRecord,
    action: DestructiveAction,
) -> Result<(), OwnershipError> {
    if record.mode != EnvironmentMode::Managed {
        return Err(OwnershipError::ExternalEnvironment);
    }
    if !matches!(
        record.state,
        EnvironmentState::Ready | EnvironmentState::Error
    ) {
        return Err(
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
        );
    }

    let (env_id, instance_id, nonce, confirmed) = match action {
        DestructiveAction::Reset {
            env_id,
            instance_id,
            ownership_nonce,
            confirmed,
        }
        | DestructiveAction::Delete {
            env_id,
            instance_id,
            ownership_nonce,
            confirmed,
        } => (env_id, instance_id, ownership_nonce, confirmed),
    };
    if env_id != record.env_id {
        return Err(OwnershipError::EnvironmentMismatch);
    }
    if instance_id != record.instance_id {
        return Err(OwnershipError::InstanceMismatch);
    }
    if nonce != record.ownership_nonce {
        return Err(OwnershipError::OwnershipMismatch);
    }
    if !confirmed {
        return Err(OwnershipError::ConfirmationRequired);
    }
    Ok(())
}
