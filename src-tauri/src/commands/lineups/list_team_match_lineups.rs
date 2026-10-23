use crate::{bootstrap::AppState, commands::parse_uuid};
use football_domain::TeamMatchLineupHistoryItem;
use tauri::State;

#[tauri::command]
pub async fn list_team_match_lineups(
    state: State<'_, AppState>,
    team_id: String,
    limit: u32,
) -> Result<Vec<TeamMatchLineupHistoryItem>, String> {
    state
        .service
        .list_team_match_lineups(parse_uuid(&team_id, "球队 ID")?, limit)
        .await
        .map_err(|error| error.to_string())
}
