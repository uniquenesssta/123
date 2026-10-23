use crate::{bootstrap::AppState, commands::parse_uuid};
use football_domain::{TeamProfileDraft, TeamProfileRecord};
use tauri::State;

#[tauri::command]
pub async fn upsert_team_profile(
    state: State<'_, AppState>,
    team_id: String,
    draft: TeamProfileDraft,
) -> Result<TeamProfileRecord, String> {
    let team_id = parse_uuid(&team_id, "球队 ID")?;
    state
        .service
        .upsert_team_profile(team_id, draft)
        .await
        .map_err(|error| error.to_string())
}
