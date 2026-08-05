use super::error;
use crate::{
    issue_log::IssueLogStore, openai_profiles::OpenAiProfileStore, runtime_log::RuntimeLogStore,
    workspace_state::WorkspaceStateStore,
};
use football_application::ApplicationService;
use football_research_gateway::CancellationToken;
use std::{collections::HashMap, path::PathBuf, sync::Arc};
use tauri::Manager;
use tokio::sync::Mutex;

pub struct AppState {
    pub service: Arc<ApplicationService>,
    pub config_path: PathBuf,
    pub issue_log: Arc<IssueLogStore>,
    pub runtime_log: Arc<RuntimeLogStore>,
    pub openai_profiles: Arc<OpenAiProfileStore>,
    pub workspace_state: Arc<WorkspaceStateStore>,
    pub api_workspace_requests: Arc<Mutex<HashMap<String, CancellationToken>>>,
}

pub(crate) fn install<R: tauri::Runtime>(app: &mut tauri::App<R>) -> Result<(), std::io::Error> {
    let config_dir = app.path().app_config_dir().map_err(error::io_error)?;
    let runtime_log = Arc::new(RuntimeLogStore::discover(&config_dir));
    let _ = runtime_log.record(
        "info",
        "application",
        "application_started",
        None,
        serde_json::json!({
            "config_directory": config_dir.display().to_string(),
            "runtime_log_path": runtime_log.path().display().to_string(),
            "runtime_log_relative_path": runtime_log.relative_display_path(),
            "runtime_log_relative_directory": r".\logs",
            "runtime_log_session_id": runtime_log.session_id(),
        }),
    );
    app.manage(AppState {
        service: Arc::new(ApplicationService::new()),
        config_path: config_dir.join("database.json"),
        issue_log: Arc::new(IssueLogStore::new(config_dir.join("issue-log.json"))),
        runtime_log,
        openai_profiles: Arc::new(OpenAiProfileStore::new(
            config_dir.join("openai-profiles.json"),
        )),
        workspace_state: Arc::new(WorkspaceStateStore::new(
            config_dir.join("workspace-state.json"),
        )),
        api_workspace_requests: Arc::new(Mutex::new(HashMap::new())),
    });
    Ok(())
}
