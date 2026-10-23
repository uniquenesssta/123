use crate::{bootstrap::AppState, commands::parse_uuid};
use tauri::State;

#[tauri::command]
pub async fn delete_match(state: State<'_, AppState>, match_id: String) -> Result<(), String> {
    let match_id = parse_uuid(&match_id, "比赛 ID")?;
    state
        .service
        .delete_match(match_id)
        .await
        .map_err(|error| error.to_string())
}
