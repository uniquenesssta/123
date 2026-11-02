use crate::bootstrap::AppState;
use tauri::State;

#[tauri::command]
pub async fn analytics_overview(
    state: State<'_, AppState>,
    snapshot_id: Option<uuid::Uuid>,
) -> Result<football_domain::AnalyticsOverview, String> {
    let result = match snapshot_id {
        Some(id) => state.service.analytics_snapshot(id).await,
        None => state.service.analytics_overview().await,
    };
    result.map_err(|error| error.to_string())
}
