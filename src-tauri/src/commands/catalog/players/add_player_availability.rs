use crate::bootstrap::AppState;
use football_domain::{PlayerAvailabilityDraft, PlayerAvailabilityRecord};
use tauri::State;

#[tauri::command]
pub async fn add_player_availability(
    state: State<'_, AppState>,
    draft: PlayerAvailabilityDraft,
) -> Result<PlayerAvailabilityRecord, String> {
    state
        .service
        .add_player_availability(draft)
        .await
        .map_err(|error| error.to_string())
}
