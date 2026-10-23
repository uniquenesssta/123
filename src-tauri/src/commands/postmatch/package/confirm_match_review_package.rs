use crate::bootstrap::AppState;
use tauri::State;

#[tauri::command]
pub async fn confirm_match_review_package(
    state: State<'_, AppState>,
    request: football_domain::MatchReviewPackageConfirmationRequest,
) -> Result<football_domain::MatchReviewPackageWorkflowRecord, String> {
    state
        .service
        .confirm_match_review_package(request)
        .await
        .map_err(|error| error.to_string())
}
