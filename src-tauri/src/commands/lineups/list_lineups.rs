use crate::{bootstrap::AppState, commands::parse_uuid};
use football_domain::LineupRecord;
use tauri::State;

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
