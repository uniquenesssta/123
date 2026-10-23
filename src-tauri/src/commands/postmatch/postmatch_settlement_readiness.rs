use crate::{bootstrap::AppState, commands::parse_uuid};
use tauri::State;

#[tauri::command]
pub async fn postmatch_settlement_readiness(
    state: State<'_, AppState>,
    match_review_id: String,
) -> Result<football_domain::PostmatchSettlementReadiness, String> {
    state
        .service
        .postmatch_settlement_readiness(parse_uuid(&match_review_id, "复盘 ID")?)
        .await
        .map_err(|error| error.to_string())
}
