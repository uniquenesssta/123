use crate::bootstrap::AppState;
use football_domain::{PlayerDynamicTagDraft, PlayerDynamicTagRecord};
use tauri::State;

#[tauri::command]
pub async fn add_player_dynamic_tag(
    state: State<'_, AppState>,
    draft: PlayerDynamicTagDraft,
) -> Result<PlayerDynamicTagRecord, String> {
    state
        .service
        .add_player_dynamic_tag(draft)
        .await
        .map_err(|error| error.to_string())
}
