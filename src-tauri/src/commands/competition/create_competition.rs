use crate::bootstrap::AppState;
use football_domain::{CompetitionDraft, CompetitionRecord};
use tauri::State;

#[tauri::command]
pub async fn create_competition(
    state: State<'_, AppState>,
    draft: CompetitionDraft,
) -> Result<CompetitionRecord, String> {
    state
        .service
        .create_competition(draft)
        .await
        .map_err(|error| error.to_string())
}
