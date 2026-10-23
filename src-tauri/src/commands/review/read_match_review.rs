use crate::{bootstrap::AppState, commands::parse_uuid};
use tauri::State;

#[tauri::command]
pub async fn read_match_review(
    state: State<'_, AppState>,
    review_id: String,
) -> Result<football_domain::MatchReviewDetail, String> {
    let review_id = parse_uuid(&review_id, "复盘 ID")?;
    state
        .service
        .read_match_review(review_id)
        .await
        .map_err(|error| error.to_string())
}
