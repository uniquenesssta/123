use crate::{bootstrap::AppState, commands::parse_uuid};
use tauri::State;

#[tauri::command]
pub async fn list_ability_candidates(
    state: State<'_, AppState>,
    status: Option<football_domain::AbilityCandidateStatus>,
    limit: u32,
    match_review_id: Option<String>,
) -> Result<Vec<football_domain::AbilityUpdateCandidateRecord>, String> {
    let match_review_id = match_review_id
        .map(|value| parse_uuid(&value, "复盘 ID"))
        .transpose()?;
    state
        .service
        .list_ability_candidates(status, limit, match_review_id)
        .await
        .map_err(|error| error.to_string())
}
