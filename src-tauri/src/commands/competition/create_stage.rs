use crate::bootstrap::AppState;
use football_domain::{StageDraft, StageRecord};
use tauri::State;

#[tauri::command]
pub async fn create_stage(
    state: State<'_, AppState>,
    draft: StageDraft,
) -> Result<StageRecord, String> {
    state
        .service
        .create_stage(draft)
        .await
        .map_err(|error| error.to_string())
}
