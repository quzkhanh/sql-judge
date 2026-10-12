use app_storage::{EnvironmentStore, HistoryStore, SubmissionRecord, TestRecord};
use environment_manager::{
    DestructiveAction, EnvironmentLedger, EnvironmentMode, EnvironmentRecord, EnvironmentState,
};
use judge_core::{Column, LogicalType, OutputPolicy, QueryLimits, RowOrder, Verdict};
use judge_postgres::{
    CancellationToken, JudgeCase, ManagedEnvironment, ManagedPostgresRuntime, PostgresSession,
    judge_submission_with_token,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};
use std::time::Instant;
use tauri::webview::PageLoadEvent;
use tauri::{Manager, State};
use tauri_plugin_log::{Target, TargetKind};
use tauri_plugin_opener::OpenerExt;
// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct JudgeRunRequest {
    problem_id: String,
    sql: String,
    mode: String,
    #[serde(default)]
    request_id: Option<String>,
}

static CANCELLATIONS: OnceLock<Mutex<HashMap<String, CancellationToken>>> = OnceLock::new();

fn cancellation_registry() -> &'static Mutex<HashMap<String, CancellationToken>> {
    CANCELLATIONS.get_or_init(|| Mutex::new(HashMap::new()))
}

struct CancellationRegistration {
    id: Option<String>,
    token: CancellationToken,
}

impl Drop for CancellationRegistration {
    fn drop(&mut self) {
        if let Some(id) = &self.id
            && let Ok(mut registry) = cancellation_registry().lock()
        {
            let owns_entry = registry
                .get(id)
                .is_some_and(|token| token.same_instance(&self.token));
            if owns_entry {
                registry.remove(id);
            }
        }
    }
}

fn register_cancellation(request_id: Option<&str>) -> CancellationRegistration {
    let token = CancellationToken::default();
    let id = request_id
        .map(str::trim)
        .filter(|id| !id.is_empty())
        .map(str::to_owned);
    if let Some(id) = &id
        && let Ok(mut registry) = cancellation_registry().lock()
    {
        registry.insert(id.clone(), token.clone());
    }
    CancellationRegistration { id, token }
}

fn cancel_judge_value(request_id: &str) -> bool {
    cancellation_registry()
        .lock()
        .ok()
        .and_then(|registry| registry.get(request_id).cloned())
        .is_some_and(|token| {
            token.cancel();
            true
        })
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct JudgeRunResult {
    status: String,
    verdict: Option<String>,
    columns: Vec<JudgeColumn>,
    rows: Vec<Vec<serde_json::Value>>,
    duration_ms: u64,
    message: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct JudgeColumn {
    name: String,
    r#type: String,
}

const SUPPORTED_PROBLEM_ID: &str = "above-average";
const SCHEMA_SQL: &str = include_str!("../../examples/packs/hr-sql-101/schema/001_tables.sql");
const REFERENCE_SQL: &str =
    include_str!("../../examples/packs/hr-sql-101/problems/above-average/reference.sql");
const BASIC_FIXTURE_SQL: &str =
    include_str!("../../examples/packs/hr-sql-101/problems/above-average/tests/basic.sql");
const EDGE_FIXTURE_SQL: &str =
    include_str!("../../examples/packs/hr-sql-101/problems/above-average/tests/edge.sql");
const EMPTY_FIXTURE_SQL: &str =
    include_str!("../../examples/packs/hr-sql-101/problems/above-average/tests/empty.sql");

struct ManagedEnvironmentState {
    store: EnvironmentStore,
    history: HistoryStore,
    ledger: EnvironmentLedger,
    runtime_root: PathBuf,
    runtime_instance_id: String,
    runtime: Option<ManagedPostgresRuntime>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CreateEnvironmentRequest {
    env_id: String,
    pack_id: String,
    namespace: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CloneEnvironmentRequest {
    source_env_id: String,
    env_id: String,
    namespace: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ResetEnvironmentRequest {
    env_id: String,
    instance_id: String,
    namespace: String,
    confirmed: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct DeleteEnvironmentRequest {
    env_id: String,
    instance_id: String,
    confirmed: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ManagedRuntimeStatus {
    available: bool,
    state: String,
    message: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ManagedEnvironmentSummary {
    env_id: String,
    instance_id: String,
    pack_id: String,
    namespace: String,
    mode: String,
    state: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct SubmissionHistorySummary {
    submission_id: String,
    pack_id: String,
    pack_version: String,
    pack_hash: String,
    engine_version: String,
    policy_version: String,
    verdict: String,
    created_at: String,
}

fn load_environment_state(path: &Path) -> Result<ManagedEnvironmentState, String> {
    let mut store = EnvironmentStore::open(path)
        .map_err(|_| "Managed environment state could not be opened.".to_owned())?;
    let history = HistoryStore::open(path)
        .map_err(|_| "Submission history could not be opened.".to_owned())?;
    let runtime_instance_id = store
        .get_or_create_runtime_instance_id()
        .map_err(|_| "Managed runtime identity could not be persisted.".to_owned())?;
    let mut records = store
        .list_environments()
        .map_err(|_| "Managed environment state could not be read.".to_owned())?;
    for record in &mut records {
        if matches!(
            record.state,
            EnvironmentState::Provisioning
                | EnvironmentState::Running
                | EnvironmentState::Rebuilding
                | EnvironmentState::Deleting
        ) {
            record.state = EnvironmentState::NeedsRecovery;
            store
                .save_environment(record)
                .map_err(|_| "Managed environment recovery state could not be saved.".to_owned())?;
        }
    }
    let ledger = EnvironmentLedger::restore(records)
        .map_err(|_| "Managed environment state failed policy validation.".to_owned())?;
    let runtime_root = path
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join("postgres-runtime");
    Ok(ManagedEnvironmentState {
        store,
        history,
        ledger,
        runtime_root,
        runtime_instance_id,
        runtime: None,
    })
}

fn managed_runtime_status_value(available: bool) -> ManagedRuntimeStatus {
    if available {
        ManagedRuntimeStatus {
            available: true,
            state: "running".to_owned(),
            message: "Managed PostgreSQL runtime is available.".to_owned(),
        }
    } else {
        ManagedRuntimeStatus {
            available: false,
            state: "unavailable".to_owned(),
            message: "Managed PostgreSQL runtime has not been started.".to_owned(),
        }
    }
}

fn ensure_managed_runtime(
    state: &mut ManagedEnvironmentState,
) -> Result<ManagedRuntimeStatus, String> {
    if state.runtime.is_none() {
        let runtime =
            ManagedPostgresRuntime::start(&state.runtime_root, &state.runtime_instance_id)
                .map_err(|_| "Managed PostgreSQL runtime could not be started.".to_owned())?;
        state.runtime = Some(runtime);
    }
    Ok(managed_runtime_status_value(true))
}

fn ownership_nonce(kind: &str, env_id: &str) -> String {
    format!(
        "nonce-{kind}-{env_id}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |duration| duration.as_nanos())
    )
}

fn environment_summary(record: &EnvironmentRecord) -> Result<ManagedEnvironmentSummary, String> {
    managed_environment_summary(record)
        .ok_or_else(|| "Only managed environments can be returned by this command.".to_owned())
}

fn provision_record(
    state: &mut ManagedEnvironmentState,
    record: EnvironmentRecord,
) -> Result<ManagedEnvironmentSummary, String> {
    state
        .ledger
        .create(record.clone())
        .map_err(|_| "Managed environment could not be reserved.".to_owned())?;
    provision_reserved_record(state, record)
}

fn provision_reserved_record(
    state: &mut ManagedEnvironmentState,
    record: EnvironmentRecord,
) -> Result<ManagedEnvironmentSummary, String> {
    state
        .store
        .save_environment(&record)
        .map_err(|_| "Managed environment reservation could not be saved.".to_owned())?;

    let environment = ManagedEnvironment::new(
        record.env_id.clone(),
        record.instance_id.clone(),
        record.namespace.clone(),
        record.ownership_nonce.clone(),
    )
    .map_err(|_| "Managed environment identity failed policy validation.".to_owned())?;
    let connection = state
        .runtime
        .as_ref()
        .ok_or_else(|| "Managed PostgreSQL runtime is unavailable.".to_owned())?
        .connection_string();
    let mut session = PostgresSession::connect(&connection)
        .map_err(|_| "Managed PostgreSQL runtime connection failed.".to_owned())?;
    session
        .provision_managed_namespace_with_schema(&environment, "sqljudge_runner", SCHEMA_SQL)
        .map_err(|_| "Managed environment schema provisioning failed.".to_owned())?;
    state
        .ledger
        .mark_ready(&record.env_id)
        .map_err(|_| "Managed environment could not enter the ready state.".to_owned())?;
    let ready = state
        .ledger
        .inspect(&record.env_id)
        .map_err(|_| "Managed environment could not be read after provisioning.".to_owned())?
        .clone();
    state
        .store
        .save_environment(&ready)
        .map_err(|_| "Managed environment ready state could not be saved.".to_owned())?;
    environment_summary(&ready)
}

fn create_managed_environment_value(
    request: CreateEnvironmentRequest,
    state: &Mutex<ManagedEnvironmentState>,
) -> Result<ManagedEnvironmentSummary, String> {
    if request.pack_id != "hr-sql-101"
        || request.env_id.trim().is_empty()
        || request.namespace.trim().is_empty()
    {
        return Err("The requested managed pack or environment identity is invalid.".to_owned());
    }
    let mut state = state
        .lock()
        .map_err(|_| "Managed environment state is busy.".to_owned())?;
    ensure_managed_runtime(&mut state)?;
    let record = EnvironmentRecord {
        env_id: request.env_id.clone(),
        instance_id: state.runtime_instance_id.clone(),
        pack_id: request.pack_id,
        namespace: request.namespace,
        ownership_nonce: ownership_nonce("create", &request.env_id),
        mode: EnvironmentMode::Managed,
        state: EnvironmentState::Provisioning,
    };
    provision_record(&mut state, record)
}

fn clone_managed_environment_value(
    request: CloneEnvironmentRequest,
    state: &Mutex<ManagedEnvironmentState>,
) -> Result<ManagedEnvironmentSummary, String> {
    if request.source_env_id.trim().is_empty()
        || request.env_id.trim().is_empty()
        || request.namespace.trim().is_empty()
    {
        return Err("The requested managed environment identity is invalid.".to_owned());
    }
    let mut state = state
        .lock()
        .map_err(|_| "Managed environment state is busy.".to_owned())?;
    ensure_managed_runtime(&mut state)?;
    let replacement = EnvironmentRecord {
        env_id: request.env_id.clone(),
        instance_id: state.runtime_instance_id.clone(),
        pack_id: String::new(),
        namespace: request.namespace,
        ownership_nonce: ownership_nonce("clone", &request.env_id),
        mode: EnvironmentMode::Managed,
        state: EnvironmentState::Provisioning,
    };
    let record = state
        .ledger
        .clone_environment(&request.source_env_id, replacement)
        .map_err(|_| "Managed environment could not be cloned.".to_owned())?;
    provision_reserved_record(&mut state, record)
}

fn lifecycle_failure(state: &mut ManagedEnvironmentState, env_id: &str, message: &str) -> String {
    if state.ledger.mark_error(env_id).is_ok()
        && let Ok(record) = state.ledger.inspect(env_id).cloned()
    {
        let _ = state.store.save_environment(&record);
    }
    message.to_owned()
}

fn reset_managed_environment_value(
    request: ResetEnvironmentRequest,
    state: &Mutex<ManagedEnvironmentState>,
) -> Result<ManagedEnvironmentSummary, String> {
    if request.env_id.trim().is_empty() || request.namespace.trim().is_empty() {
        return Err("The requested reset identity is invalid.".to_owned());
    }
    let mut state = state
        .lock()
        .map_err(|_| "Managed environment state is busy.".to_owned())?;
    ensure_managed_runtime(&mut state)?;
    let current = state
        .ledger
        .inspect(&request.env_id)
        .map_err(|_| "Managed environment was not found.".to_owned())?
        .clone();
    if current.namespace == request.namespace {
        return Err("Reset requires a replacement namespace.".to_owned());
    }
    let action = DestructiveAction::Reset {
        env_id: request.env_id.clone(),
        instance_id: request.instance_id,
        ownership_nonce: current.ownership_nonce.clone(),
        confirmed: request.confirmed,
    };
    state
        .ledger
        .begin_rebuild(&request.env_id, action.clone())
        .map_err(|_| "Managed environment reset authorization failed.".to_owned())?;
    let rebuilding = state
        .ledger
        .inspect(&request.env_id)
        .map_err(|_| "Managed environment reset state could not be read.".to_owned())?
        .clone();
    state
        .store
        .save_environment(&rebuilding)
        .map_err(|_| "Managed environment reset state could not be saved.".to_owned())?;

    let replacement = EnvironmentRecord {
        env_id: current.env_id.clone(),
        instance_id: current.instance_id.clone(),
        pack_id: current.pack_id.clone(),
        namespace: request.namespace,
        ownership_nonce: current.ownership_nonce.clone(),
        mode: EnvironmentMode::Managed,
        state: EnvironmentState::Ready,
    };
    let replacement_environment = ManagedEnvironment::new(
        replacement.env_id.clone(),
        replacement.instance_id.clone(),
        replacement.namespace.clone(),
        replacement.ownership_nonce.clone(),
    )
    .map_err(|_| {
        lifecycle_failure(
            &mut state,
            &request.env_id,
            "Managed replacement identity failed policy validation.",
        )
    })?;
    let old_environment = ManagedEnvironment::new(
        current.env_id.clone(),
        current.instance_id.clone(),
        current.namespace.clone(),
        current.ownership_nonce.clone(),
    )
    .map_err(|_| "Managed environment identity failed policy validation.".to_owned())?;
    let connection = state
        .runtime
        .as_ref()
        .ok_or_else(|| "Managed PostgreSQL runtime is unavailable.".to_owned())?
        .connection_string();
    let mut session = PostgresSession::connect(&connection)
        .map_err(|_| "Managed PostgreSQL runtime connection failed.".to_owned())?;
    session
        .provision_managed_namespace_with_schema(
            &replacement_environment,
            "sqljudge_runner",
            SCHEMA_SQL,
        )
        .map_err(|_| "Managed replacement schema provisioning failed.".to_owned())?;
    state
        .ledger
        .finish_rebuild(&request.env_id, replacement, action, |_| true)
        .map_err(|_| "Managed replacement could not enter the ready state.".to_owned())?;
    let ready = state
        .ledger
        .inspect(&request.env_id)
        .map_err(|_| "Managed replacement state could not be read.".to_owned())?
        .clone();
    state
        .store
        .save_environment(&ready)
        .map_err(|_| "Managed replacement state could not be saved.".to_owned())?;
    if session.drop_managed_namespace(&old_environment).is_err() {
        state
            .ledger
            .mark_error(&request.env_id)
            .map_err(|_| "Managed reset cleanup failed.".to_owned())?;
        let error_state = state
            .ledger
            .inspect(&request.env_id)
            .map_err(|_| "Managed reset cleanup state could not be read.".to_owned())?
            .clone();
        state
            .store
            .save_environment(&error_state)
            .map_err(|_| "Managed reset cleanup state could not be saved.".to_owned())?;
        return Err("Managed reset replacement succeeded but old cleanup failed.".to_owned());
    }
    environment_summary(&ready)
}

fn delete_managed_environment_value(
    request: DeleteEnvironmentRequest,
    state: &Mutex<ManagedEnvironmentState>,
) -> Result<(), String> {
    if request.env_id.trim().is_empty() {
        return Err("The requested delete identity is invalid.".to_owned());
    }
    let mut state = state
        .lock()
        .map_err(|_| "Managed environment state is busy.".to_owned())?;
    ensure_managed_runtime(&mut state)?;
    let current = state
        .ledger
        .inspect(&request.env_id)
        .map_err(|_| "Managed environment was not found.".to_owned())?
        .clone();
    let action = DestructiveAction::Delete {
        env_id: request.env_id.clone(),
        instance_id: request.instance_id,
        ownership_nonce: current.ownership_nonce.clone(),
        confirmed: request.confirmed,
    };
    state
        .ledger
        .begin_delete(&request.env_id, action.clone())
        .map_err(|_| "Managed environment delete authorization failed.".to_owned())?;
    let deleting = state
        .ledger
        .inspect(&request.env_id)
        .map_err(|_| "Managed environment delete state could not be read.".to_owned())?
        .clone();
    state
        .store
        .save_environment(&deleting)
        .map_err(|_| "Managed environment delete state could not be saved.".to_owned())?;
    let environment = ManagedEnvironment::new(
        current.env_id.clone(),
        current.instance_id.clone(),
        current.namespace.clone(),
        current.ownership_nonce.clone(),
    )
    .map_err(|_| "Managed environment identity failed policy validation.".to_owned())?;
    let connection = state
        .runtime
        .as_ref()
        .ok_or_else(|| "Managed PostgreSQL runtime is unavailable.".to_owned())?
        .connection_string();
    let mut session = PostgresSession::connect(&connection)
        .map_err(|_| "Managed PostgreSQL runtime connection failed.".to_owned())?;
    if session.drop_managed_namespace(&environment).is_err() {
        state
            .ledger
            .mark_error(&request.env_id)
            .map_err(|_| "Managed delete cleanup failed.".to_owned())?;
        let error_state = state
            .ledger
            .inspect(&request.env_id)
            .map_err(|_| "Managed delete cleanup state could not be read.".to_owned())?
            .clone();
        state
            .store
            .save_environment(&error_state)
            .map_err(|_| "Managed delete cleanup state could not be saved.".to_owned())?;
        return Err("Managed environment cleanup failed.".to_owned());
    }
    state
        .ledger
        .finish_delete(&request.env_id, action)
        .map_err(|_| "Managed environment could not be removed from the ledger.".to_owned())?;
    state
        .store
        .delete_environment(&request.env_id)
        .map_err(|_| "Managed environment could not be removed from storage.".to_owned())?;
    Ok(())
}

fn managed_environment_summary(record: &EnvironmentRecord) -> Option<ManagedEnvironmentSummary> {
    if record.mode != EnvironmentMode::Managed {
        return None;
    }
    Some(ManagedEnvironmentSummary {
        env_id: record.env_id.clone(),
        instance_id: record.instance_id.clone(),
        pack_id: record.pack_id.clone(),
        namespace: record.namespace.clone(),
        mode: match record.mode {
            EnvironmentMode::Managed => "managed".to_owned(),
            EnvironmentMode::ExternalExplorer => "external_explorer".to_owned(),
        },
        state: match record.state {
            EnvironmentState::Provisioning => "provisioning".to_owned(),
            EnvironmentState::Ready => "ready".to_owned(),
            EnvironmentState::Running => "running".to_owned(),
            EnvironmentState::Rebuilding => "rebuilding".to_owned(),
            EnvironmentState::Deleting => "deleting".to_owned(),
            EnvironmentState::NeedsRecovery => "needs_recovery".to_owned(),
            EnvironmentState::Error => "error".to_owned(),
        },
    })
}

fn policy_error(message: &str) -> JudgeRunResult {
    JudgeRunResult {
        status: "error".to_owned(),
        verdict: Some("PolicyViolation".to_owned()),
        columns: Vec::new(),
        rows: Vec::new(),
        duration_ms: 0,
        message: message.to_owned(),
    }
}

fn unavailable_judge(request: JudgeRunRequest) -> JudgeRunResult {
    if request.problem_id.trim().is_empty() || request.mode != "run" && request.mode != "submit" {
        return policy_error("The judge request is missing a valid problem or mode.");
    }
    if request.sql.len() > 65_536 {
        return policy_error("SQL input exceeds the 64 KiB limit.");
    }

    JudgeRunResult {
        status: "unavailable".to_owned(),
        verdict: None,
        columns: Vec::new(),
        rows: Vec::new(),
        duration_ms: 0,
        message: "Managed PostgreSQL runtime is not configured for this desktop.".to_owned(),
    }
}

fn verdict_name(verdict: Verdict) -> &'static str {
    match verdict {
        Verdict::Accepted => "Accepted",
        Verdict::WrongAnswer => "WrongAnswer",
        Verdict::SyntaxError => "SyntaxError",
        Verdict::RuntimeError => "RuntimeError",
        Verdict::TimeLimitExceeded => "TimeLimitExceeded",
        Verdict::ResultLimitExceeded => "ResultLimitExceeded",
        Verdict::PolicyViolation => "PolicyViolation",
        Verdict::Cancelled => "Cancelled",
        Verdict::JudgeError => "JudgeError",
    }
}

fn output_policy() -> OutputPolicy {
    OutputPolicy::new(
        RowOrder::Unordered,
        true,
        true,
        vec![
            Column::new("employee_id", LogicalType::Integer),
            Column::new("employee_name", LogicalType::Text),
        ],
    )
}

fn judge_cases(submit: bool) -> Vec<JudgeCase<'static>> {
    let fixtures = if submit {
        vec![
            ("basic", BASIC_FIXTURE_SQL),
            ("edge", EDGE_FIXTURE_SQL),
            ("empty", EMPTY_FIXTURE_SQL),
        ]
    } else {
        vec![("basic", BASIC_FIXTURE_SQL)]
    };
    fixtures
        .into_iter()
        .map(|(id, fixture_sql)| JudgeCase {
            id,
            schema_sql: SCHEMA_SQL,
            fixture_sql,
            reference_sql: REFERENCE_SQL,
            output: output_policy(),
            limits: QueryLimits::new(2_000, 10_000, 16 * 1024 * 1024),
        })
        .collect()
}

fn embedded_pack_hash() -> String {
    let mut digest = Sha256::new();
    digest.update(SCHEMA_SQL.as_bytes());
    digest.update(REFERENCE_SQL.as_bytes());
    digest.update(BASIC_FIXTURE_SQL.as_bytes());
    digest.update(EDGE_FIXTURE_SQL.as_bytes());
    digest.update(EMPTY_FIXTURE_SQL.as_bytes());
    let bytes = digest.finalize();
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn submission_id() -> String {
    format!(
        "submission-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |duration| duration.as_nanos())
    )
}

fn created_at() -> String {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or_else(
            |_| "0".to_owned(),
            |duration| duration.as_secs().to_string(),
        )
}

fn judge_request(
    request: JudgeRunRequest,
    state: &Mutex<ManagedEnvironmentState>,
    expected_mode: &str,
) -> JudgeRunResult {
    if request.problem_id.trim().is_empty()
        || request.sql.len() > 65_536
        || !matches!(request.mode.as_str(), "run" | "submit")
    {
        return unavailable_judge(request);
    }
    if request.problem_id != SUPPORTED_PROBLEM_ID || request.mode != expected_mode {
        return policy_error("The requested problem or judge mode is not available.");
    }
    let cancellation = register_cancellation(request.request_id.as_deref());
    let cancellation_token = cancellation.token.clone();
    let started = Instant::now();
    let mut state = match state.lock() {
        Ok(state) => state,
        Err(_) => {
            return JudgeRunResult {
                status: "error".to_owned(),
                verdict: Some("JudgeError".to_owned()),
                columns: Vec::new(),
                rows: Vec::new(),
                duration_ms: 0,
                message: "Managed environment state is busy.".to_owned(),
            };
        }
    };
    if state.runtime.is_none() && ensure_managed_runtime(&mut state).is_err() {
        return JudgeRunResult {
            status: "unavailable".to_owned(),
            verdict: None,
            columns: Vec::new(),
            rows: Vec::new(),
            duration_ms: started.elapsed().as_millis() as u64,
            message: "Managed PostgreSQL runtime could not be started.".to_owned(),
        };
    }
    let connection_string = match state.runtime.as_ref() {
        Some(runtime) => runtime.connection_string(),
        None => {
            return JudgeRunResult {
                status: "unavailable".to_owned(),
                verdict: None,
                columns: Vec::new(),
                rows: Vec::new(),
                duration_ms: started.elapsed().as_millis() as u64,
                message: "Managed PostgreSQL runtime is unavailable.".to_owned(),
            };
        }
    };
    let mut session = match PostgresSession::connect(&connection_string) {
        Ok(session) => session,
        Err(_) => {
            return JudgeRunResult {
                status: "error".to_owned(),
                verdict: Some("JudgeError".to_owned()),
                columns: Vec::new(),
                rows: Vec::new(),
                duration_ms: started.elapsed().as_millis() as u64,
                message: "Managed PostgreSQL runtime connection failed.".to_owned(),
            };
        }
    };
    let report = judge_submission_with_token(
        &mut session,
        "sqljudge_runner",
        &judge_cases(expected_mode == "submit"),
        &request.sql,
        &cancellation_token,
    );
    let duration_ms = started.elapsed().as_millis() as u64;
    let verdict = verdict_name(report.verdict).to_owned();
    if expected_mode == "submit" {
        let tests = report
            .tests
            .iter()
            .map(|test| TestRecord {
                test_id: test.id.clone(),
                verdict: verdict_name(test.verdict).to_owned(),
                elapsed_ms: duration_ms,
            })
            .collect::<Vec<_>>();
        let submission = SubmissionRecord {
            submission_id: submission_id(),
            pack_id: "hr-sql-101".to_owned(),
            pack_version: "1.0.0".to_owned(),
            pack_hash: embedded_pack_hash(),
            engine_version: "postgresql-managed".to_owned(),
            policy_version: "policy-v1".to_owned(),
            verdict: verdict.clone(),
            created_at: created_at(),
        };
        if state.history.save_submission(&submission, &tests).is_err() {
            return JudgeRunResult {
                status: "error".to_owned(),
                verdict: Some("JudgeError".to_owned()),
                columns: Vec::new(),
                rows: Vec::new(),
                duration_ms,
                message: "Submission completed but history could not be saved.".to_owned(),
            };
        }
    }
    JudgeRunResult {
        status: "completed".to_owned(),
        verdict: Some(verdict),
        columns: Vec::new(),
        rows: Vec::new(),
        duration_ms,
        message: format!("{} test case(s) evaluated.", report.tests.len()),
    }
}

#[tauri::command]
fn run_sample(
    request: JudgeRunRequest,
    state: State<'_, Mutex<ManagedEnvironmentState>>,
) -> JudgeRunResult {
    judge_request(request, state.inner(), "run")
}

#[tauri::command]
fn submit_solution(
    request: JudgeRunRequest,
    state: State<'_, Mutex<ManagedEnvironmentState>>,
) -> JudgeRunResult {
    judge_request(request, state.inner(), "submit")
}

#[tauri::command]
fn cancel_judge(request_id: String) -> bool {
    cancel_judge_value(&request_id)
}

#[tauri::command]
fn list_managed_environments(
    state: State<'_, Mutex<ManagedEnvironmentState>>,
) -> Result<Vec<ManagedEnvironmentSummary>, String> {
    let state = state
        .lock()
        .map_err(|_| "Managed environment state is busy.".to_owned())?;
    Ok(state
        .ledger
        .list()
        .into_iter()
        .filter_map(managed_environment_summary)
        .collect())
}

#[tauri::command]
fn inspect_managed_environment(
    env_id: String,
    state: State<'_, Mutex<ManagedEnvironmentState>>,
) -> Result<Option<ManagedEnvironmentSummary>, String> {
    if env_id.trim().is_empty() {
        return Err("Environment ID is required.".to_owned());
    }
    let state = state
        .lock()
        .map_err(|_| "Managed environment state is busy.".to_owned())?;
    Ok(state
        .ledger
        .inspect(&env_id)
        .ok()
        .and_then(managed_environment_summary))
}

#[tauri::command]
fn list_submission_history(
    state: State<'_, Mutex<ManagedEnvironmentState>>,
) -> Result<Vec<SubmissionHistorySummary>, String> {
    let state = state
        .lock()
        .map_err(|_| "Managed environment state is busy.".to_owned())?;
    state
        .history
        .list_submissions()
        .map(|records| {
            records
                .into_iter()
                .map(|record| SubmissionHistorySummary {
                    submission_id: record.submission_id,
                    pack_id: record.pack_id,
                    pack_version: record.pack_version,
                    pack_hash: record.pack_hash,
                    engine_version: record.engine_version,
                    policy_version: record.policy_version,
                    verdict: record.verdict,
                    created_at: record.created_at,
                })
                .collect()
        })
        .map_err(|_| "Submission history could not be read.".to_owned())
}

#[tauri::command]
fn create_managed_environment(
    request: CreateEnvironmentRequest,
    state: State<'_, Mutex<ManagedEnvironmentState>>,
) -> Result<ManagedEnvironmentSummary, String> {
    create_managed_environment_value(request, state.inner())
}

#[tauri::command]
fn clone_managed_environment(
    request: CloneEnvironmentRequest,
    state: State<'_, Mutex<ManagedEnvironmentState>>,
) -> Result<ManagedEnvironmentSummary, String> {
    clone_managed_environment_value(request, state.inner())
}

#[tauri::command]
fn reset_managed_environment(
    request: ResetEnvironmentRequest,
    state: State<'_, Mutex<ManagedEnvironmentState>>,
) -> Result<ManagedEnvironmentSummary, String> {
    reset_managed_environment_value(request, state.inner())
}

#[tauri::command]
fn delete_managed_environment(
    request: DeleteEnvironmentRequest,
    state: State<'_, Mutex<ManagedEnvironmentState>>,
) -> Result<(), String> {
    delete_managed_environment_value(request, state.inner())
}

#[tauri::command]
fn managed_runtime_status(
    state: State<'_, Mutex<ManagedEnvironmentState>>,
) -> Result<ManagedRuntimeStatus, String> {
    let state = state
        .lock()
        .map_err(|_| "Managed environment state is busy.".to_owned())?;
    Ok(managed_runtime_status_value(state.runtime.is_some()))
}

#[tauri::command]
fn start_managed_runtime(
    state: State<'_, Mutex<ManagedEnvironmentState>>,
) -> Result<ManagedRuntimeStatus, String> {
    let mut state = state
        .lock()
        .map_err(|_| "Managed environment state is busy.".to_owned())?;
    ensure_managed_runtime(&mut state)
}

fn external_navigation_plugin<R: tauri::Runtime>() -> tauri::plugin::TauriPlugin<R> {
    tauri::plugin::Builder::<R>::new("external-navigation")
        .on_navigation(|webview, url| {
            let is_internal_host = matches!(
                url.host_str(),
                Some("localhost") | Some("127.0.0.1") | Some("tauri.localhost") | Some("::1")
            );

            let is_internal = url.scheme() == "tauri" || is_internal_host;

            if is_internal {
                return true;
            }

            let is_external_link = matches!(url.scheme(), "http" | "https" | "mailto" | "tel");

            if is_external_link {
                log::info!(
                    "opening external link in system browser: {}://{}",
                    url.scheme(),
                    url.host_str().unwrap_or("redacted")
                );
                let _ = webview.opener().open_url(url.as_str(), None::<&str>);
                return false;
            }

            true
        })
        .build()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(
            tauri_plugin_log::Builder::new()
                .targets([
                    Target::new(TargetKind::Stdout),
                    Target::new(TargetKind::LogDir { file_name: None }),
                    Target::new(TargetKind::Webview),
                ])
                .build(),
        )
        .plugin(tauri_plugin_opener::init())
        .plugin(external_navigation_plugin())
        .setup(|app| {
            let data_dir = app
                .path()
                .app_local_data_dir()
                .map_err(Box::<dyn std::error::Error>::from)?;
            std::fs::create_dir_all(&data_dir)?;
            let state = load_environment_state(&data_dir.join("sql-judge.sqlite"))
                .map_err(std::io::Error::other)?;
            app.manage(Mutex::new(state));
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            greet,
            run_sample,
            submit_solution,
            cancel_judge,
            list_managed_environments,
            inspect_managed_environment,
            create_managed_environment,
            clone_managed_environment,
            reset_managed_environment,
            delete_managed_environment,
            list_submission_history,
            managed_runtime_status,
            start_managed_runtime
        ])
        .on_page_load(|webview, payload| {
            if webview.label() == "main" && matches!(payload.event(), PageLoadEvent::Finished) {
                log::info!("main webview finished loading");
                let _ = webview.window().show();
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use super::{
        CloneEnvironmentRequest, CreateEnvironmentRequest, DeleteEnvironmentRequest,
        JudgeRunRequest, ResetEnvironmentRequest, SUPPORTED_PROBLEM_ID, cancel_judge_value,
        clone_managed_environment_value, create_managed_environment_value,
        delete_managed_environment_value, ensure_managed_runtime, judge_request,
        load_environment_state, managed_environment_summary, managed_runtime_status_value,
        register_cancellation, reset_managed_environment_value, unavailable_judge, verdict_name,
    };
    use std::sync::Mutex;

    #[test]
    fn native_judge_cancellation_registry_is_scoped_to_request_id() {
        let registration = register_cancellation(Some("request-1"));
        assert!(cancel_judge_value("request-1"));
        assert!(registration.token.is_cancelled());
        drop(registration);
        assert!(!cancel_judge_value("request-1"));
    }

    #[test]
    fn native_bridge_fails_closed_when_runtime_is_missing() {
        let result = unavailable_judge(JudgeRunRequest {
            problem_id: "above-average".to_owned(),
            sql: "SELECT 1".to_owned(),
            mode: "run".to_owned(),
            request_id: None,
        });

        assert_eq!(result.status, "unavailable");
        assert!(result.verdict.is_none());
    }

    #[test]
    fn native_runtime_status_is_unavailable_before_explicit_start() {
        let status = managed_runtime_status_value(false);
        assert!(!status.available);
        assert_eq!(status.state, "unavailable");
    }

    #[test]
    fn native_verdict_names_are_stable() {
        assert_eq!(verdict_name(judge_core::Verdict::Accepted), "Accepted");
        assert_eq!(
            verdict_name(judge_core::Verdict::WrongAnswer),
            "WrongAnswer"
        );
    }

    #[test]
    fn native_run_executes_sample_against_managed_runtime() {
        let root = std::env::temp_dir().join(format!(
            "sql-judge-tauri-run-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system clock")
                .as_nanos()
        ));
        std::fs::create_dir(&root).expect("create test root");
        let path = root.join("state.sqlite");
        let mut native_state = load_environment_state(&path).expect("load native state");
        ensure_managed_runtime(&mut native_state).expect("start managed runtime");
        let state = Mutex::new(native_state);
        let result = judge_request(
            JudgeRunRequest {
                problem_id: SUPPORTED_PROBLEM_ID.to_owned(),
                sql: "SELECT e.id AS employee_id, e.name AS employee_name FROM employees AS e WHERE e.salary > (SELECT AVG(peer.salary) FROM employees AS peer WHERE peer.department_id = e.department_id)".to_owned(),
                mode: "run".to_owned(),
                request_id: None,
            },
            &state,
            "run",
        );

        assert_eq!(result.status, "completed");
        assert_eq!(result.verdict.as_deref(), Some("Accepted"));
        let submit = judge_request(
            JudgeRunRequest {
                problem_id: SUPPORTED_PROBLEM_ID.to_owned(),
                sql: "SELECT 1".to_owned(),
                mode: "submit".to_owned(),
                request_id: None,
            },
            &state,
            "submit",
        );
        assert_eq!(submit.status, "completed");
        assert_eq!(submit.verdict.as_deref(), Some("WrongAnswer"));
        let state_guard = state.lock().expect("lock history state");
        let history = state_guard
            .history
            .list_submissions()
            .expect("list submission history");
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].pack_id, "hr-sql-101");
        assert!(!history[0].pack_hash.is_empty());
        drop(state_guard);
        drop(state);
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn native_create_and_clone_provision_isolated_ready_environments() {
        let root = std::env::temp_dir().join(format!(
            "sql-judge-tauri-environments-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system clock")
                .as_nanos()
        ));
        std::fs::create_dir(&root).expect("create test root");
        let state = Mutex::new(
            load_environment_state(&root.join("state.sqlite")).expect("load native state"),
        );
        let first = create_managed_environment_value(
            CreateEnvironmentRequest {
                env_id: "env-a".to_owned(),
                pack_id: "hr-sql-101".to_owned(),
                namespace: "sj_env_a".to_owned(),
            },
            &state,
        )
        .expect("create first environment");
        let second = clone_managed_environment_value(
            CloneEnvironmentRequest {
                source_env_id: "env-a".to_owned(),
                env_id: "env-b".to_owned(),
                namespace: "sj_env_b".to_owned(),
            },
            &state,
        )
        .expect("clone second environment");

        assert_eq!(first.state, "ready");
        assert_eq!(second.state, "ready");
        assert_ne!(first.namespace, second.namespace);
        let reset = reset_managed_environment_value(
            ResetEnvironmentRequest {
                env_id: first.env_id.clone(),
                instance_id: first.instance_id.clone(),
                namespace: "sj_env_reset".to_owned(),
                confirmed: true,
            },
            &state,
        )
        .expect("reset first environment");
        assert_eq!(reset.state, "ready");
        delete_managed_environment_value(
            DeleteEnvironmentRequest {
                env_id: second.env_id.clone(),
                instance_id: second.instance_id.clone(),
                confirmed: true,
            },
            &state,
        )
        .expect("delete second environment");
        let state_guard = state.lock().expect("lock native state");
        assert_eq!(state_guard.ledger.list().len(), 1);
        assert_eq!(
            state_guard
                .ledger
                .inspect(&first.env_id)
                .expect("reset environment")
                .namespace,
            "sj_env_reset"
        );
        drop(state_guard);
        drop(state);
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn native_reset_failure_persists_error_without_replacing_old_environment() {
        let root = std::env::temp_dir().join(format!(
            "sql-judge-tauri-reset-error-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system clock")
                .as_nanos()
        ));
        std::fs::create_dir(&root).expect("create test root");
        let state = Mutex::new(
            load_environment_state(&root.join("state.sqlite")).expect("load native state"),
        );
        let created = create_managed_environment_value(
            CreateEnvironmentRequest {
                env_id: "env-a".to_owned(),
                pack_id: "hr-sql-101".to_owned(),
                namespace: "sj_env_a".to_owned(),
            },
            &state,
        )
        .expect("create environment");
        let result = reset_managed_environment_value(
            ResetEnvironmentRequest {
                env_id: created.env_id.clone(),
                instance_id: created.instance_id,
                namespace: "public".to_owned(),
                confirmed: true,
            },
            &state,
        );

        assert!(result.is_err());
        let state_guard = state.lock().expect("lock native state");
        assert_eq!(
            state_guard
                .ledger
                .inspect("env-a")
                .expect("environment")
                .state,
            environment_manager::EnvironmentState::Error
        );
        assert_eq!(
            state_guard
                .ledger
                .inspect("env-a")
                .expect("environment")
                .namespace,
            "sj_env_a"
        );
        drop(state_guard);
        drop(state);
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn native_bridge_rejects_oversized_sql_before_runtime_access() {
        let result = unavailable_judge(JudgeRunRequest {
            problem_id: "above-average".to_owned(),
            sql: "x".repeat(65_537),
            mode: "submit".to_owned(),
            request_id: None,
        });

        assert_eq!(result.status, "error");
        assert_eq!(result.verdict.as_deref(), Some("PolicyViolation"));
    }

    #[test]
    fn native_environment_state_restores_persisted_records() {
        use app_storage::EnvironmentStore;
        use environment_manager::{EnvironmentMode, EnvironmentRecord, EnvironmentState};

        let path = std::env::temp_dir().join(format!(
            "sql-judge-tauri-environment-state-{}.sqlite",
            std::process::id()
        ));
        let _ = std::fs::remove_file(&path);
        let mut store = EnvironmentStore::open(&path).expect("open environment store");
        store
            .save_environment(&EnvironmentRecord {
                env_id: "env-a".to_owned(),
                instance_id: "instance-a".to_owned(),
                pack_id: "hr-sql-101".to_owned(),
                namespace: "sj_env_a".to_owned(),
                ownership_nonce: "nonce-a".to_owned(),
                mode: EnvironmentMode::Managed,
                state: EnvironmentState::NeedsRecovery,
            })
            .expect("save environment");

        let state = load_environment_state(&path).expect("restore native state");
        assert_eq!(
            state
                .ledger
                .inspect("env-a")
                .expect("restored environment")
                .state,
            EnvironmentState::NeedsRecovery
        );
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn native_environment_state_marks_interrupted_operations_for_recovery() {
        use app_storage::EnvironmentStore;
        use environment_manager::{EnvironmentMode, EnvironmentRecord, EnvironmentState};

        let path = std::env::temp_dir().join(format!(
            "sql-judge-tauri-environment-recovery-{}.sqlite",
            std::process::id()
        ));
        let _ = std::fs::remove_file(&path);
        let mut store = EnvironmentStore::open(&path).expect("open environment store");
        store
            .save_environment(&EnvironmentRecord {
                env_id: "env-rebuild".to_owned(),
                instance_id: "instance-rebuild".to_owned(),
                pack_id: "hr-sql-101".to_owned(),
                namespace: "sj_env_rebuild".to_owned(),
                ownership_nonce: "nonce-rebuild".to_owned(),
                mode: EnvironmentMode::Managed,
                state: EnvironmentState::Rebuilding,
            })
            .expect("save interrupted environment");
        store
            .save_environment(&EnvironmentRecord {
                env_id: "env-running".to_owned(),
                instance_id: "instance-running".to_owned(),
                pack_id: "hr-sql-101".to_owned(),
                namespace: "sj_env_running".to_owned(),
                ownership_nonce: "nonce-running".to_owned(),
                mode: EnvironmentMode::Managed,
                state: EnvironmentState::Running,
            })
            .expect("save running environment");

        let state = load_environment_state(&path).expect("restore native state");
        assert_eq!(
            state
                .ledger
                .inspect("env-rebuild")
                .expect("restored environment")
                .state,
            EnvironmentState::NeedsRecovery
        );
        let persisted = EnvironmentStore::open(&path)
            .expect("reopen environment store")
            .get_environment("env-rebuild")
            .expect("read recovered environment")
            .expect("environment exists");
        assert_eq!(persisted.state, EnvironmentState::NeedsRecovery);
        let running = EnvironmentStore::open(&path)
            .expect("reopen environment store")
            .get_environment("env-running")
            .expect("read recovered running environment")
            .expect("running environment exists");
        assert_eq!(running.state, EnvironmentState::NeedsRecovery);
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn managed_environment_views_exclude_external_records() {
        use environment_manager::{EnvironmentMode, EnvironmentRecord, EnvironmentState};

        let external = EnvironmentRecord {
            env_id: "external-a".to_owned(),
            instance_id: "instance-external".to_owned(),
            pack_id: "pack-a".to_owned(),
            namespace: "sj_external".to_owned(),
            ownership_nonce: "not-owned".to_owned(),
            mode: EnvironmentMode::ExternalExplorer,
            state: EnvironmentState::Ready,
        };
        assert!(managed_environment_summary(&external).is_none());
    }
}
