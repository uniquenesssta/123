use crate::bootstrap::AppState;
use football_domain::{
    FormationDistributionQuery, FormationRecord, FormationUsageDistributionDraft,
    FormationUsageDistributionRecord, FormationUsageListQuery,
};
use tauri::State;

#[tauri::command]
pub async fn list_formations(
    state: State<'_, AppState>,
    active_only: bool,
) -> Result<Vec<FormationRecord>, String> {
    state
        .service
        .list_formations(active_only)
        .await
        .map_err(|error| error.to_string())
}

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

#[tauri::command]
pub async fn resolve_formation_distribution(
    state: State<'_, AppState>,
    query: FormationDistributionQuery,
) -> Result<football_domain::ResolvedFormationDistribution, String> {
    state
        .service
        .resolve_formation_distribution(query)
        .await
        .map_err(|error| error.to_string())
}
