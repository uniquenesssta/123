use crate::bootstrap::AppState;
use tauri::State;

#[tauri::command]
pub async fn decide_evidence_scoring_item(
    state: State<'_, AppState>,
    draft: football_domain::EvidenceScoringDecisionDraft,
) -> Result<football_domain::EvidenceScoringItemRecord, String> {
    state
        .service
        .decide_evidence_scoring_item(draft)
        .await
        .map_err(|error| error.to_string())
}
