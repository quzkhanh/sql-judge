use app_storage::EnvironmentStore;
use environment_manager::{
    EnvironmentLedger, EnvironmentMode, EnvironmentRecord, EnvironmentState,
};
use judge_core::{Column, LogicalType, OutputPolicy, QueryLimits, RowOrder, Verdict};
use judge_postgres::{
    JudgeCase, ManagedEnvironment, ManagedPostgresRuntime, PostgresSession, judge_submission,
};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::Mutex;
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

fn load_environment_state(path: &Path) -> Result<ManagedEnvironmentState, String> {
    let mut store = EnvironmentStore::open(path)
        .map_err(|_| "Managed environment state could not be opened.".to_owned())?;
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
    let report = judge_submission(
        &mut session,
        "sqljudge_runner",
        &judge_cases(expected_mode == "submit"),
        &request.sql,
    );
    JudgeRunResult {
        status: "completed".to_owned(),
        verdict: Some(verdict_name(report.verdict).to_owned()),
        columns: Vec::new(),
        rows: Vec::new(),
        duration_ms: started.elapsed().as_millis() as u64,
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
            list_managed_environments,
            inspect_managed_environment,
            create_managed_environment,
            clone_managed_environment,
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
        CloneEnvironmentRequest, CreateEnvironmentRequest, JudgeRunRequest, SUPPORTED_PROBLEM_ID,
        clone_managed_environment_value, create_managed_environment_value, ensure_managed_runtime,
        judge_request, load_environment_state, managed_environment_summary,
        managed_runtime_status_value, unavailable_judge, verdict_name,
    };
    use std::sync::Mutex;

    #[test]
    fn native_bridge_fails_closed_when_runtime_is_missing() {
        let result = unavailable_judge(JudgeRunRequest {
            problem_id: "above-average".to_owned(),
            sql: "SELECT 1".to_owned(),
            mode: "run".to_owned(),
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
            },
            &state,
            "submit",
        );
        assert_eq!(submit.status, "completed");
        assert_eq!(submit.verdict.as_deref(), Some("WrongAnswer"));
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
        let state_guard = state.lock().expect("lock native state");
        assert_eq!(state_guard.ledger.list().len(), 2);
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
