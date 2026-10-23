use crate::bootstrap::AppState;
use tauri::State;

#[tauri::command]
pub async fn decide_data_quality_finding(
    state: State<'_, AppState>,
    draft: football_domain::DataQualityDecisionDraft,
) -> Result<football_domain::DataQualityFinding, String> {
    state
        .service
        .decide_data_quality_finding(draft)
        .await
        .map_err(|error| error.to_string())
}
