use crate::bootstrap::AppState;
use tauri::State;

#[tauri::command]
pub async fn commit_match_review_package(
    state: State<'_, AppState>,
    request: football_domain::MatchReviewPackageCommitRequest,
) -> Result<football_domain::MatchReviewPackageCommitResult, String> {
    state
        .service
        .commit_match_review_package(request)
        .await
        .map_err(|error| error.to_string())
}
