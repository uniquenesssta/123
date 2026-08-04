use super::{parse_uuid, AppState};
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
#[tauri::command]
pub async fn list_reviewable_matches(
    state: State<'_, AppState>,
    limit: u32,
) -> Result<Vec<football_domain::ReviewableMatch>, String> {
    state
        .service
        .list_reviewable_matches(limit)
        .await
        .map_err(|error| error.to_string())
}
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
#[tauri::command]
pub async fn list_ability_candidates(
    state: State<'_, AppState>,
    status: Option<football_domain::AbilityCandidateStatus>,
    limit: u32,
    match_review_id: Option<String>,
) -> Result<Vec<football_domain::AbilityUpdateCandidateRecord>, String> {
    let match_review_id = match_review_id
        .map(|value| parse_uuid(&value, "复盘 ID"))
        .transpose()?;
    state
        .service
        .list_ability_candidates(status, limit, match_review_id)
        .await
        .map_err(|error| error.to_string())
}
#[tauri::command]
pub async fn decide_ability_candidate(
    state: State<'_, AppState>,
    draft: football_domain::AbilityCandidateDecisionDraft,
) -> Result<football_domain::AbilityUpdateCandidateRecord, String> {
    state
        .service
        .decide_ability_candidate(draft)
        .await
        .map_err(|error| error.to_string())
}
