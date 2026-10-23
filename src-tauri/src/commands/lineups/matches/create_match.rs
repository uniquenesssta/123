use crate::bootstrap::AppState;
use football_domain::{MatchDraft, MatchRecord};
use tauri::State;

#[tauri::command]
pub async fn create_match(
    state: State<'_, AppState>,
    draft: MatchDraft,
) -> Result<MatchRecord, String> {
    state
        .service
        .create_match(draft)
        .await
        .map_err(|error| error.to_string())
}
