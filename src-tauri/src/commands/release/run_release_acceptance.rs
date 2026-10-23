use crate::bootstrap::AppState;
use tauri::State;

#[tauri::command]
pub async fn run_release_acceptance(
    state: State<'_, AppState>,
    request: football_domain::ReleaseAcceptanceRequest,
) -> Result<football_domain::ReleaseAcceptanceRun, String> {
    state
        .service
        .run_release_acceptance(request)
        .await
        .map_err(|error| error.to_string())
}
