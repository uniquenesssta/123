use crate::{bootstrap::AppState, commands::parse_uuid};
use tauri::State;

#[tauri::command]
pub async fn preview_match_review_package(
    state: State<'_, AppState>,
    input_path: String,
    expected_match_id: Option<String>,
) -> Result<football_domain::MatchReviewPackagePreview, String> {
    let expected_match_id = expected_match_id
        .as_deref()
        .map(|value| parse_uuid(value, "当前比赛 ID"))
        .transpose()?;
    state
        .service
        .preview_match_review_package(input_path, expected_match_id)
        .await
        .map_err(|error| error.to_string())
}
