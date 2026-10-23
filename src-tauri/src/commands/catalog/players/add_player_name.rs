use crate::bootstrap::AppState;
use football_domain::{PlayerNameDraft, PlayerNameRecord};
use tauri::State;

#[tauri::command]
pub async fn add_player_name(
    state: State<'_, AppState>,
    draft: PlayerNameDraft,
) -> Result<PlayerNameRecord, String> {
    state
        .service
        .add_player_name(draft)
        .await
        .map_err(|error| error.to_string())
}
