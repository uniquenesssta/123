use crate::bootstrap::AppState;
use football_domain::{FormationUsageDistributionDraft, FormationUsageDistributionRecord};
use tauri::State;

#[tauri::command]
pub async fn save_formation_usage_distribution(
    state: State<'_, AppState>,
    draft: FormationUsageDistributionDraft,
) -> Result<FormationUsageDistributionRecord, String> {
    state
        .service
        .save_formation_usage_distribution(draft)
        .await
        .map_err(|error| error.to_string())
}
