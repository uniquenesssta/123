use crate::{bootstrap::AppState, commands::parse_uuid};
use tauri::State;

#[tauri::command]
pub async fn generate_match_review_from_package(
    state: State<'_, AppState>,
    package_id: String,
) -> Result<football_domain::MatchReviewPackageReviewResult, String> {
    state
        .service
        .generate_match_review_from_package(parse_uuid(&package_id, "资料包 ID")?)
        .await
        .map_err(|error| error.to_string())
}
