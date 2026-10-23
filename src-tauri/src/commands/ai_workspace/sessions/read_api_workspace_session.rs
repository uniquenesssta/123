use crate::{bootstrap::AppState, commands::parse_uuid};
use football_domain::ApiWorkspaceSessionDetail;
use tauri::State;

#[tauri::command]
pub async fn read_api_workspace_session(
    state: State<'_, AppState>,
    session_id: String,
) -> Result<ApiWorkspaceSessionDetail, String> {
    let session_id = parse_uuid(&session_id, "API协作会话 ID")?;
    state
        .service
        .read_api_workspace_session(session_id)
        .await
        .map_err(|error| error.to_string())
}
