use crate::bootstrap::AppState;
use football_domain::{LineupPairDraft, LineupPairRecord};
use tauri::State;

#[tauri::command]
pub async fn create_lineup_pair(
    state: State<'_, AppState>,
    draft: LineupPairDraft,
) -> Result<LineupPairRecord, String> {
    state
        .service
        .create_lineup_pair(draft)
        .await
        .map_err(|error| error.to_string())
}
