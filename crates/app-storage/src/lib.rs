use environment_manager::{EnvironmentMode, EnvironmentRecord, EnvironmentState};
use rusqlite::{Connection, OptionalExtension, params};
use std::path::Path;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum StorageError {
    #[error("SQLite operation failed")]
    Sqlite(#[from] rusqlite::Error),
    #[error("invalid history record: {0}")]
    InvalidRecord(&'static str),
    #[error("invalid environment record: {0}")]
    InvalidEnvironment(&'static str),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubmissionRecord {
    pub submission_id: String,
    pub pack_id: String,
    pub pack_version: String,
    pub pack_hash: String,
    pub engine_version: String,
    pub policy_version: String,
    pub verdict: String,
    pub created_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TestRecord {
    pub test_id: String,
    pub verdict: String,
    pub elapsed_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SubmissionSummary {
    pub submission_id: String,
    pub pack_id: String,
    pub pack_version: String,
    pub pack_hash: String,
    pub engine_version: String,
    pub policy_version: String,
    pub verdict: String,
    pub created_at: String,
}

pub struct HistoryStore {
    connection: Connection,
}

pub struct EnvironmentStore {
    connection: Connection,
}

impl EnvironmentStore {
    pub fn open(path: impl AsRef<Path>) -> Result<Self, StorageError> {
        let connection = Connection::open(path)?;
        connection.pragma_update(None, "journal_mode", "WAL")?;
        connection.execute_batch(
            "CREATE TABLE IF NOT EXISTS environment_ledger (
                 env_id TEXT PRIMARY KEY NOT NULL,
                 instance_id TEXT NOT NULL,
                 pack_id TEXT NOT NULL,
                 namespace TEXT NOT NULL UNIQUE,
                 ownership_nonce TEXT NOT NULL,
                 mode TEXT NOT NULL CHECK (mode IN ('managed', 'external_explorer')),
                 state TEXT NOT NULL CHECK (state IN (
                     'provisioning', 'ready', 'running', 'rebuilding',
                     'deleting', 'needs_recovery', 'error'
                 ))
             );
             CREATE INDEX IF NOT EXISTS idx_environment_pack ON environment_ledger(pack_id);",
        )?;
        Ok(Self { connection })
    }

    pub fn save_environment(
        &mut self,
        environment: &EnvironmentRecord,
    ) -> Result<(), StorageError> {
        validate_environment(environment)?;
        let transaction = self.connection.transaction()?;
        transaction.execute(
            "INSERT INTO environment_ledger (
                 env_id, instance_id, pack_id, namespace, ownership_nonce, mode, state
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
             ON CONFLICT(env_id) DO UPDATE SET
                 instance_id = excluded.instance_id,
                 pack_id = excluded.pack_id,
                 namespace = excluded.namespace,
                 ownership_nonce = excluded.ownership_nonce,
                 mode = excluded.mode,
                 state = excluded.state",
            params![
                environment.env_id,
                environment.instance_id,
                environment.pack_id,
                environment.namespace,
                environment.ownership_nonce,
                environment_mode_name(environment.mode),
                environment_state_name(environment.state),
            ],
        )?;
        transaction.commit()?;
        Ok(())
    }

    pub fn get_environment(&self, env_id: &str) -> Result<Option<EnvironmentRecord>, StorageError> {
        if env_id.trim().is_empty() {
            return Err(StorageError::InvalidEnvironment(
                "environment id must be non-empty",
            ));
        }
        let mut statement = self.connection.prepare(
            "SELECT env_id, instance_id, pack_id, namespace, ownership_nonce, mode, state
             FROM environment_ledger WHERE env_id = ?1",
        )?;
        let row = statement
            .query_row([env_id], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, String>(4)?,
                    row.get::<_, String>(5)?,
                    row.get::<_, String>(6)?,
                ))
            })
            .optional()?;

        row.map(decode_environment).transpose()
    }

    pub fn list_environments(&self) -> Result<Vec<EnvironmentRecord>, StorageError> {
        let mut statement = self.connection.prepare(
            "SELECT env_id, instance_id, pack_id, namespace, ownership_nonce, mode, state
             FROM environment_ledger ORDER BY env_id",
        )?;
        let rows = statement.query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, String>(4)?,
                row.get::<_, String>(5)?,
                row.get::<_, String>(6)?,
            ))
        })?;

        let mut environments = Vec::new();
        for row in rows {
            environments.push(decode_environment(row?)?);
        }
        Ok(environments)
    }
}

fn validate_environment(environment: &EnvironmentRecord) -> Result<(), StorageError> {
    [
        (&environment.env_id, "environment id"),
        (&environment.instance_id, "instance id"),
        (&environment.pack_id, "pack id"),
        (&environment.namespace, "namespace"),
        (&environment.ownership_nonce, "ownership nonce"),
    ]
    .iter()
    .find(|(value, _)| value.trim().is_empty())
    .map_or_else(
        || {
            if !is_owned_namespace(&environment.namespace) {
                Err(StorageError::InvalidEnvironment("namespace"))
            } else {
                Ok(())
            }
        },
        |(_, field)| Err(StorageError::InvalidEnvironment(field)),
    )
}

fn is_owned_namespace(namespace: &str) -> bool {
    namespace.len() <= 63
        && namespace.starts_with("sj_")
        && namespace
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
}

fn environment_mode_name(mode: EnvironmentMode) -> &'static str {
    match mode {
        EnvironmentMode::Managed => "managed",
        EnvironmentMode::ExternalExplorer => "external_explorer",
    }
}

fn environment_state_name(state: EnvironmentState) -> &'static str {
    match state {
        EnvironmentState::Provisioning => "provisioning",
        EnvironmentState::Ready => "ready",
        EnvironmentState::Running => "running",
        EnvironmentState::Rebuilding => "rebuilding",
        EnvironmentState::Deleting => "deleting",
        EnvironmentState::NeedsRecovery => "needs_recovery",
        EnvironmentState::Error => "error",
    }
}

fn decode_environment(
    (env_id, instance_id, pack_id, namespace, ownership_nonce, mode, state): (
        String,
        String,
        String,
        String,
        String,
        String,
        String,
    ),
) -> Result<EnvironmentRecord, StorageError> {
    let mode = match mode.as_str() {
        "managed" => EnvironmentMode::Managed,
        "external_explorer" => EnvironmentMode::ExternalExplorer,
        _ => return Err(StorageError::InvalidEnvironment("unknown environment mode")),
    };
    let state = match state.as_str() {
        "provisioning" => EnvironmentState::Provisioning,
        "ready" => EnvironmentState::Ready,
        "running" => EnvironmentState::Running,
        "rebuilding" => EnvironmentState::Rebuilding,
        "deleting" => EnvironmentState::Deleting,
        "needs_recovery" => EnvironmentState::NeedsRecovery,
        "error" => EnvironmentState::Error,
        _ => {
            return Err(StorageError::InvalidEnvironment(
                "unknown environment state",
            ));
        }
    };
    Ok(EnvironmentRecord {
        env_id,
        instance_id,
        pack_id,
        namespace,
        ownership_nonce,
        mode,
        state,
    })
}

impl HistoryStore {
    pub fn open(path: impl AsRef<Path>) -> Result<Self, StorageError> {
        let connection = Connection::open(path)?;
        connection.pragma_update(None, "journal_mode", "WAL")?;
        connection.execute_batch(
            "CREATE TABLE IF NOT EXISTS schema_version (
                 version INTEGER PRIMARY KEY CHECK (version = 1)
             );
             INSERT OR IGNORE INTO schema_version(version) VALUES (1);
             CREATE TABLE IF NOT EXISTS submissions (
                 submission_id TEXT PRIMARY KEY NOT NULL,
                 pack_id TEXT NOT NULL,
                 pack_version TEXT NOT NULL,
                 pack_hash TEXT NOT NULL,
                 engine_version TEXT NOT NULL,
                 policy_version TEXT NOT NULL,
                 verdict TEXT NOT NULL,
                 created_at TEXT NOT NULL
             );
             CREATE TABLE IF NOT EXISTS submission_tests (
                 submission_id TEXT NOT NULL REFERENCES submissions(submission_id) ON DELETE CASCADE,
                 test_id TEXT NOT NULL,
                 verdict TEXT NOT NULL,
                 elapsed_ms INTEGER NOT NULL CHECK (elapsed_ms >= 0),
                 PRIMARY KEY (submission_id, test_id)
             );
             CREATE INDEX IF NOT EXISTS idx_submissions_created_at ON submissions(created_at);",
        )?;
        Ok(Self { connection })
    }

    pub fn save_submission(
        &mut self,
        submission: &SubmissionRecord,
        tests: &[TestRecord],
    ) -> Result<(), StorageError> {
        validate_submission(submission)?;
        for test in tests {
            if test.test_id.trim().is_empty() || test.verdict.trim().is_empty() {
                return Err(StorageError::InvalidRecord("test fields must be non-empty"));
            }
        }

        let transaction = self.connection.transaction()?;
        transaction.execute(
            "INSERT INTO submissions (
                 submission_id, pack_id, pack_version, pack_hash,
                 engine_version, policy_version, verdict, created_at
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                submission.submission_id,
                submission.pack_id,
                submission.pack_version,
                submission.pack_hash,
                submission.engine_version,
                submission.policy_version,
                submission.verdict,
                submission.created_at,
            ],
        )?;
        for test in tests {
            let elapsed_ms = i64::try_from(test.elapsed_ms)
                .map_err(|_| StorageError::InvalidRecord("elapsed time is too large"))?;
            transaction.execute(
                "INSERT INTO submission_tests (submission_id, test_id, verdict, elapsed_ms)
                 VALUES (?1, ?2, ?3, ?4)",
                params![
                    submission.submission_id,
                    test.test_id,
                    test.verdict,
                    elapsed_ms,
                ],
            )?;
        }
        transaction.commit()?;
        Ok(())
    }

    pub fn list_submissions(&self) -> Result<Vec<SubmissionSummary>, StorageError> {
        let mut statement = self.connection.prepare(
            "SELECT submission_id, pack_id, pack_version, pack_hash,
                    engine_version, policy_version, verdict, created_at
             FROM submissions ORDER BY created_at DESC, submission_id DESC",
        )?;
        let rows = statement.query_map([], |row| {
            Ok(SubmissionSummary {
                submission_id: row.get(0)?,
                pack_id: row.get(1)?,
                pack_version: row.get(2)?,
                pack_hash: row.get(3)?,
                engine_version: row.get(4)?,
                policy_version: row.get(5)?,
                verdict: row.get(6)?,
                created_at: row.get(7)?,
            })
        })?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(StorageError::from)
    }

    pub fn test_count(&self, submission_id: &str) -> Result<usize, StorageError> {
        let count: i64 = self.connection.query_row(
            "SELECT COUNT(*) FROM submission_tests WHERE submission_id = ?1",
            [submission_id],
            |row| row.get(0),
        )?;
        Ok(count as usize)
    }
}

fn validate_submission(submission: &SubmissionRecord) -> Result<(), StorageError> {
    let fields = [
        (&submission.submission_id, "submission id"),
        (&submission.pack_id, "pack id"),
        (&submission.pack_version, "pack version"),
        (&submission.pack_hash, "pack hash"),
        (&submission.engine_version, "engine version"),
        (&submission.policy_version, "policy version"),
        (&submission.verdict, "verdict"),
        (&submission.created_at, "created at"),
    ];
    fields
        .iter()
        .find(|(value, _)| value.trim().is_empty())
        .map_or(Ok(()), |(_, field)| Err(StorageError::InvalidRecord(field)))
}
