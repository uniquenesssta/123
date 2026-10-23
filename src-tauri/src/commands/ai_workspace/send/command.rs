use super::workflow;
use crate::{bootstrap::AppState, commands::SendApiWorkspaceCommand};
use football_domain::ApiWorkspaceSessionDetail;
use tauri::State;

#[tauri::command]
pub async fn send_api_workspace_message(
    state: State<'_, AppState>,
    command: SendApiWorkspaceCommand,
) -> Result<ApiWorkspaceSessionDetail, String> {
    workflow::execute(&state, command).await
}
