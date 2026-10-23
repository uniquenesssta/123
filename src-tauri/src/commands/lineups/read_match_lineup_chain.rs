use crate::{bootstrap::AppState, commands::parse_uuid};
use football_domain::MatchLineupChain;
use tauri::State;

#[tauri::command]
pub async fn read_match_lineup_chain(
    state: State<'_, AppState>,
    match_id: String,
    snapshot_type: String,
) -> Result<MatchLineupChain, String> {
    state
        .service
        .read_match_lineup_chain(parse_uuid(&match_id, "比赛 ID")?, snapshot_type)
        .await
        .map_err(|error| error.to_string())
}
