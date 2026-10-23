use crate::bootstrap::AppState;
use football_domain::{RoundDraft, RoundRecord};
use tauri::State;

#[tauri::command]
pub async fn create_round(
    state: State<'_, AppState>,
    draft: RoundDraft,
) -> Result<RoundRecord, String> {
    state
        .service
        .create_round(draft)
        .await
        .map_err(|error| error.to_string())
}
