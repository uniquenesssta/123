use crate::{bootstrap::AppState, commands::parse_uuid};
use tauri::State;

#[tauri::command]
pub async fn export_match_review_package(
    state: State<'_, AppState>,
    output_path: String,
    match_id: String,
) -> Result<football_domain::MatchReviewPackageSummary, String> {
    state
        .service
        .export_match_review_package(output_path, parse_uuid(&match_id, "比赛 ID")?)
        .await
        .map_err(|error| error.to_string())
}
