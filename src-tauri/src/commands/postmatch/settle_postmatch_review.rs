use crate::bootstrap::AppState;
use tauri::State;

#[tauri::command]
pub async fn settle_postmatch_review(
    state: State<'_, AppState>,
    draft: football_domain::PostmatchSettlementDraft,
) -> Result<football_domain::PostmatchSettlementRecord, String> {
    state
        .service
        .settle_postmatch_review(draft)
        .await
        .map_err(|error| error.to_string())
}
