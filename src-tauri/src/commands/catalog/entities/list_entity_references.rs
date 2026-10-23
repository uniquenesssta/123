use crate::bootstrap::AppState;
use football_domain::{EntityReferenceQuery, EntityReferenceRecord};
use tauri::State;

#[tauri::command]
pub async fn list_entity_references(
    state: State<'_, AppState>,
    query: EntityReferenceQuery,
) -> Result<Vec<EntityReferenceRecord>, String> {
    state
        .service
        .list_entity_references(query)
        .await
        .map_err(|error| error.to_string())
}
