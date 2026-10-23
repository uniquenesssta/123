use crate::{bootstrap::AppState, commands::parse_uuid};
use football_domain::BulkArchiveResult;
use tauri::State;

#[tauri::command]
pub async fn bulk_archive_entities(
    state: State<'_, AppState>,
    entity_type: String,
    entity_ids: Vec<String>,
) -> Result<BulkArchiveResult, String> {
    let ids = entity_ids
        .iter()
        .map(|value| parse_uuid(value, "实体 ID"))
        .collect::<Result<Vec<_>, _>>()?;
    state
        .service
        .bulk_archive_entities(entity_type, ids)
        .await
        .map_err(|error| error.to_string())
}
