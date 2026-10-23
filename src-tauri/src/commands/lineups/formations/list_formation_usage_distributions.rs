use crate::bootstrap::AppState;
use football_domain::{FormationUsageDistributionRecord, FormationUsageListQuery};
use tauri::State;

#[tauri::command]
pub async fn list_formation_usage_distributions(
    state: State<'_, AppState>,
    query: FormationUsageListQuery,
) -> Result<Vec<FormationUsageDistributionRecord>, String> {
    state
        .service
        .list_formation_usage_distributions(query)
        .await
        .map_err(|error| error.to_string())
}
