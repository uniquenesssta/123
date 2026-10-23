use crate::{bootstrap::AppState, commands::parse_uuid};
use tauri::State;

#[tauri::command]
pub async fn list_parameter_promotion_decisions(
    state: State<'_, AppState>,
    candidate_id: String,
) -> Result<Vec<football_domain::ParameterPromotionDecisionRecord>, String> {
    state
        .service
        .list_parameter_promotion_decisions(parse_uuid(&candidate_id, "候选 ID")?)
        .await
        .map_err(|error| error.to_string())
}
