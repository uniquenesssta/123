use crate::bootstrap::AppState;
use football_domain::{PlayerDraft, PlayerRecord};
use tauri::State;

#[tauri::command]
pub async fn create_player(
    state: State<'_, AppState>,
    draft: PlayerDraft,
) -> Result<PlayerRecord, String> {
    state
        .service
        .create_player(draft)
        .await
        .map_err(|error| error.to_string())
}
