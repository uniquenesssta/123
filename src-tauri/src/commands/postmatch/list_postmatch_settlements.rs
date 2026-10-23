use crate::bootstrap::AppState;
use tauri::State;

#[tauri::command]
pub async fn list_postmatch_settlements(
    state: State<'_, AppState>,
    limit: u32,
) -> Result<Vec<football_domain::PostmatchSettlementRecord>, String> {
    state
        .service
        .list_postmatch_settlements(limit)
        .await
        .map_err(|error| error.to_string())
}
