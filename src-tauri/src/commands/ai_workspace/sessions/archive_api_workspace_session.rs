use crate::{bootstrap::AppState, commands::parse_uuid};
use tauri::State;

#[tauri::command]
pub async fn archive_api_workspace_session(
    state: State<'_, AppState>,
    session_id: String,
) -> Result<(), String> {
    let session_id = parse_uuid(&session_id, "AI问答会话 ID")?;
    state
        .service
        .archive_api_workspace_session(session_id)
        .await
        .map_err(|error| error.to_string())
}
