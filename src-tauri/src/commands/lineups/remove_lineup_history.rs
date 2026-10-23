use crate::{bootstrap::AppState, commands::parse_uuid};
use football_domain::LineupHistoryRemovalResult;
use tauri::State;

#[tauri::command]
pub async fn remove_lineup_history(
    state: State<'_, AppState>,
    lineup_id: String,
    reason: Option<String>,
) -> Result<LineupHistoryRemovalResult, String> {
    state
        .service
        .remove_lineup_history(parse_uuid(&lineup_id, "阵容 ID")?, reason)
        .await
        .map_err(|error| error.to_string())
}
