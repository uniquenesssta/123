use crate::bootstrap::AppState;
use football_domain::{CompetitionBindingDraft, CompetitionBindingSummary};
use tauri::State;

#[tauri::command]
pub async fn create_competition_binding(
    state: State<'_, AppState>,
    draft: CompetitionBindingDraft,
) -> Result<CompetitionBindingSummary, String> {
    state
        .service
        .create_competition_binding(draft)
        .await
        .map_err(|error| error.to_string())
}
