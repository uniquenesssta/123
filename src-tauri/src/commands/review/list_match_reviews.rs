use crate::bootstrap::AppState;
use tauri::State;

#[tauri::command]
pub async fn list_match_reviews(
    state: State<'_, AppState>,
    limit: u32,
) -> Result<Vec<football_domain::MatchReviewSummary>, String> {
    state
        .service
        .list_match_reviews(limit)
        .await
        .map_err(|error| error.to_string())
}
