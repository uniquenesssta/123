use crate::bootstrap::AppState;
use tauri::State;

#[tauri::command]
pub async fn analytics_overview(
    state: State<'_, AppState>,
) -> Result<football_domain::AnalyticsOverview, String> {
    state
        .service
        .analytics_overview()
        .await
        .map_err(|error| error.to_string())
}
