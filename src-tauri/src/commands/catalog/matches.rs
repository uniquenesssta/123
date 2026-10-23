use crate::{bootstrap::AppState, commands::parse_uuid};
use football_domain::{MatchDraft, MatchRecord};
use tauri::State;

#[tauri::command]
pub async fn create_match(
    state: State<'_, AppState>,
    draft: MatchDraft,
) -> Result<MatchRecord, String> {
    state
        .service
        .create_match(draft)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn delete_match(state: State<'_, AppState>, match_id: String) -> Result<(), String> {
    let match_id = parse_uuid(&match_id, "比赛 ID")?;
    state
        .service
        .delete_match(match_id)
        .await
        .map_err(|error| error.to_string())
}
