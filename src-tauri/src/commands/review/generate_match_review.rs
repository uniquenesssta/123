use crate::bootstrap::AppState;
use tauri::State;

#[tauri::command]
pub async fn generate_match_review(
    state: State<'_, AppState>,
    draft: football_domain::MatchReviewDraft,
) -> Result<football_domain::MatchReviewDetail, String> {
    state
        .service
        .generate_match_review(draft)
        .await
        .map_err(|error| error.to_string())
}
