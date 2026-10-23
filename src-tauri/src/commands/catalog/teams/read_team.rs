use crate::{bootstrap::AppState, commands::parse_uuid};
use football_domain::TeamDetail;
use tauri::State;

#[tauri::command]
pub async fn read_team(state: State<'_, AppState>, team_id: String) -> Result<TeamDetail, String> {
    let team_id = parse_uuid(&team_id, "球队 ID")?;
    state
        .service
        .read_team(team_id)
        .await
        .map_err(|error| error.to_string())
}
