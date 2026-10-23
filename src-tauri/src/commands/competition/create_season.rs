use crate::bootstrap::AppState;
use football_domain::{SeasonDraft, SeasonRecord};
use tauri::State;

#[tauri::command]
pub async fn create_season(
    state: State<'_, AppState>,
    draft: SeasonDraft,
) -> Result<SeasonRecord, String> {
    state
        .service
        .create_season(draft)
        .await
        .map_err(|error| error.to_string())
}
