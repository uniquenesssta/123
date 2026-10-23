use crate::bootstrap::AppState;
use football_domain::{LineupDraft, LineupRecord};
use tauri::State;

#[tauri::command]
pub async fn create_lineup(
    state: State<'_, AppState>,
    draft: LineupDraft,
) -> Result<LineupRecord, String> {
    state
        .service
        .create_lineup(draft)
        .await
        .map_err(|error| error.to_string())
}
