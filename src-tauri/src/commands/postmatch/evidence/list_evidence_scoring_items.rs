use crate::bootstrap::AppState;
use tauri::State;

#[tauri::command]
pub async fn list_evidence_scoring_items(
    state: State<'_, AppState>,
    status: Option<String>,
    limit: u32,
) -> Result<Vec<football_domain::EvidenceScoringItemRecord>, String> {
    state
        .service
        .list_evidence_scoring_items(status, limit)
        .await
        .map_err(|error| error.to_string())
}
