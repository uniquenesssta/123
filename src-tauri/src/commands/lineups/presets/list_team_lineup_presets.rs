use crate::{bootstrap::AppState, commands::parse_uuid};
use football_domain::TeamLineupPresetRecord;
use tauri::State;

#[tauri::command]
pub async fn list_team_lineup_presets(
    state: State<'_, AppState>,
    team_id: String,
    include_archived: bool,
) -> Result<Vec<TeamLineupPresetRecord>, String> {
    let team_id = parse_uuid(&team_id, "球队 ID")?;
    state
        .service
        .list_team_lineup_presets(team_id, include_archived)
        .await
        .map_err(|error| error.to_string())
}
