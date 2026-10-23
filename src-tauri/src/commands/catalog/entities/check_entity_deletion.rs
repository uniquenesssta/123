use crate::{bootstrap::AppState, commands::parse_uuid};
use football_domain::EntityDeletionCheck;
use tauri::State;

#[tauri::command]
pub async fn check_entity_deletion(
    state: State<'_, AppState>,
    entity_type: String,
    entity_id: String,
) -> Result<EntityDeletionCheck, String> {
    let entity_id = parse_uuid(&entity_id, "实体 ID")?;
    state
        .service
        .check_entity_deletion(entity_type, entity_id)
        .await
        .map_err(|error| error.to_string())
}
