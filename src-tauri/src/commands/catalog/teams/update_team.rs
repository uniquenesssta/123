use crate::{bootstrap::AppState, commands::parse_uuid};
use football_domain::{TeamDraft, TeamRecord};
use tauri::State;

#[tauri::command]
pub async fn update_team(
    state: State<'_, AppState>,
    team_id: String,
    draft: TeamDraft,
) -> Result<TeamRecord, String> {
    let team_id = parse_uuid(&team_id, "球队 ID")?;
    state
        .service
        .update_team(team_id, draft)
        .await
        .map_err(|error| error.to_string())
}
