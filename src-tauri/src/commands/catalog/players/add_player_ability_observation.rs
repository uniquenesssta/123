use crate::bootstrap::AppState;
use football_domain::{PlayerAbilityObservationDraft, PlayerAbilityObservationRecord};
use tauri::State;

#[tauri::command]
pub async fn add_player_ability_observation(
    state: State<'_, AppState>,
    draft: PlayerAbilityObservationDraft,
) -> Result<PlayerAbilityObservationRecord, String> {
    state
        .service
        .add_player_ability_observation(draft)
        .await
        .map_err(|error| error.to_string())
}
