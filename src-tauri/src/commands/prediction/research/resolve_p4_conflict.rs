use crate::bootstrap::AppState;
use football_domain::{P4TaskWorkspace, ResolveP4ConflictCommand};
use tauri::State;

#[tauri::command]
pub async fn resolve_p4_conflict(
    state: State<'_, AppState>,
    command: ResolveP4ConflictCommand,
) -> Result<P4TaskWorkspace, String> {
    state
        .service
        .resolve_p4_conflict(command)
        .await
        .map_err(|error| error.to_string())
}
