use crate::bootstrap::AppState;
use football_domain::FormationDistributionQuery;
use tauri::State;

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
