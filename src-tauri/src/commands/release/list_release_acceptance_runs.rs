use crate::bootstrap::AppState;
use tauri::State;

#[tauri::command]
pub async fn list_release_acceptance_runs(
    state: State<'_, AppState>,
    limit: u32,
) -> Result<Vec<football_domain::ReleaseAcceptanceRunSummary>, String> {
    state
        .service
        .list_release_acceptance_runs(limit)
        .await
        .map_err(|error| error.to_string())
}
