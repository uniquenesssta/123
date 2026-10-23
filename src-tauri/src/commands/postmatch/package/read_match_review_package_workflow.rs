use crate::{bootstrap::AppState, commands::parse_uuid};
use tauri::State;

#[tauri::command]
pub async fn read_match_review_package_workflow(
    state: State<'_, AppState>,
    match_id: String,
) -> Result<Option<football_domain::MatchReviewPackageWorkflowRecord>, String> {
    state
        .service
        .read_match_review_package_workflow(parse_uuid(&match_id, "比赛 ID")?)
        .await
        .map_err(|error| error.to_string())
}
