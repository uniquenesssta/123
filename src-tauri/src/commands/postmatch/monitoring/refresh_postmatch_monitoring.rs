use crate::bootstrap::AppState;
use tauri::State;

#[tauri::command]
pub async fn refresh_postmatch_monitoring(
    state: State<'_, AppState>,
    request: football_domain::PostmatchMonitoringRequest,
) -> Result<football_domain::PostmatchOverview, String> {
    state
        .service
        .refresh_postmatch_monitoring(request)
        .await
        .map_err(|error| error.to_string())
}
