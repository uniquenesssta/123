use crate::{bootstrap::AppState, commands::parse_uuid};
use tauri::State;

#[tauri::command]
pub async fn commit_match_review_package_facts(
    state: State<'_, AppState>,
    package_id: String,
) -> Result<football_domain::MatchReviewPackageFactsCommitResult, String> {
    state
        .service
        .commit_match_review_package_facts(parse_uuid(&package_id, "资料包 ID")?)
        .await
        .map_err(|error| error.to_string())
}
