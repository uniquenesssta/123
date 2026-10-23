use crate::{bootstrap::AppState, commands::parse_uuid};
use football_domain::{
    LineupDraft, LineupHistoryRemovalResult, LineupPairDraft, LineupPairRecord, LineupRecord,
    MatchLineupChain, TeamMatchLineupHistoryItem,
};
use tauri::State;

#[tauri::command]
pub async fn create_lineup(
    state: State<'_, AppState>,
    draft: LineupDraft,
) -> Result<LineupRecord, String> {
    state
        .service
        .create_lineup(draft)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn create_lineup_pair(
    state: State<'_, AppState>,
    draft: LineupPairDraft,
) -> Result<LineupPairRecord, String> {
    state
        .service
        .create_lineup_pair(draft)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn list_lineups(
    state: State<'_, AppState>,
    match_id: Option<String>,
    limit: u32,
) -> Result<Vec<LineupRecord>, String> {
    let match_id = match_id
        .map(|value| parse_uuid(&value, "比赛 ID"))
        .transpose()?;
    state
        .service
        .list_lineups(match_id, limit)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn read_lineup(
    state: State<'_, AppState>,
    lineup_id: String,
) -> Result<LineupRecord, String> {
    let lineup_id = parse_uuid(&lineup_id, "阵容 ID")?;
    state
        .service
        .read_lineup(lineup_id)
        .await
        .map_err(|error| error.to_string())
}

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
