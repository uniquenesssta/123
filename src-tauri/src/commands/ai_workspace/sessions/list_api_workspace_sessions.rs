use crate::bootstrap::AppState;
use football_domain::ApiWorkspaceSessionRecord;
use tauri::State;

#[tauri::command]
pub async fn list_api_workspace_sessions(
    state: State<'_, AppState>,
    limit: u32,
) -> Result<Vec<ApiWorkspaceSessionRecord>, String> {
    state
        .service
        .list_api_workspace_sessions(limit)
        .await
        .map_err(|error| error.to_string())
}
