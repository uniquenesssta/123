use crate::bootstrap::AppState;
use football_domain::PlayerCatalogReferenceData;
use tauri::State;

#[tauri::command]
pub async fn player_catalog_reference_data(
    state: State<'_, AppState>,
) -> Result<PlayerCatalogReferenceData, String> {
    state
        .service
        .player_catalog_reference_data()
        .await
        .map_err(|error| error.to_string())
}
