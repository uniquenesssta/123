use crate::bootstrap::AppState;
use football_domain::{PlayerPositionDraft, PlayerPositionRecord};
use tauri::State;

#[tauri::command]
pub async fn assign_player_position(
    state: State<'_, AppState>,
    draft: PlayerPositionDraft,
) -> Result<PlayerPositionRecord, String> {
    state
        .service
        .assign_player_position(draft)
        .await
        .map_err(|error| error.to_string())
}
