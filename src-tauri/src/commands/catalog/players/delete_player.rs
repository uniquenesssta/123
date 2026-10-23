use crate::{bootstrap::AppState, commands::parse_uuid};
use tauri::State;

#[tauri::command]
pub async fn delete_player(state: State<'_, AppState>, player_id: String) -> Result<(), String> {
    let player_id = parse_uuid(&player_id, "球员 ID")?;
    state
        .service
        .delete_player(player_id)
        .await
        .map_err(|error| error.to_string())
}
