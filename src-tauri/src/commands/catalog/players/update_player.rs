use crate::{bootstrap::AppState, commands::parse_uuid};
use football_domain::{PlayerDraft, PlayerRecord};
use tauri::State;

#[tauri::command]
pub async fn update_player(
    state: State<'_, AppState>,
    player_id: String,
    draft: PlayerDraft,
) -> Result<PlayerRecord, String> {
    let parsed = parse_uuid(&player_id, "球员 ID")?;
    state
        .service
        .update_player(parsed, draft)
        .await
        .map_err(|error| error.to_string())
}
