use crate::{bootstrap::AppState, commands::parse_uuid};
use football_domain::PlayerDetail;
use tauri::State;

#[tauri::command]
pub async fn read_player(
    state: State<'_, AppState>,
    player_id: String,
) -> Result<PlayerDetail, String> {
    let player_id = parse_uuid(&player_id, "球员 ID")?;
    state
        .service
        .read_player(player_id)
        .await
        .map_err(|error| error.to_string())
}
