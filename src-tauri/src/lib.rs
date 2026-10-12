use app_storage::EnvironmentStore;
use environment_manager::{
    EnvironmentLedger, EnvironmentMode, EnvironmentRecord, EnvironmentState,
};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::Mutex;
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

struct ManagedEnvironmentState {
    _store: EnvironmentStore,
    ledger: EnvironmentLedger,
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
    Ok(ManagedEnvironmentState {
        _store: store,
        ledger,
    })
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

fn unavailable_judge(request: JudgeRunRequest) -> JudgeRunResult {
    if request.problem_id.trim().is_empty() || request.mode != "run" && request.mode != "submit" {
        return JudgeRunResult {
            status: "error".to_owned(),
            verdict: Some("PolicyViolation".to_owned()),
            columns: Vec::new(),
            rows: Vec::new(),
            duration_ms: 0,
            message: "The judge request is missing a valid problem or mode.".to_owned(),
        };
    }
    if request.sql.len() > 65_536 {
        return JudgeRunResult {
            status: "error".to_owned(),
            verdict: Some("PolicyViolation".to_owned()),
            columns: Vec::new(),
            rows: Vec::new(),
            duration_ms: 0,
            message: "SQL input exceeds the 64 KiB limit.".to_owned(),
        };
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

#[tauri::command]
fn run_sample(request: JudgeRunRequest) -> JudgeRunResult {
    unavailable_judge(request)
}

#[tauri::command]
fn submit_solution(request: JudgeRunRequest) -> JudgeRunResult {
    unavailable_judge(request)
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
            inspect_managed_environment
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
        JudgeRunRequest, load_environment_state, managed_environment_summary, unavailable_judge,
    };

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
