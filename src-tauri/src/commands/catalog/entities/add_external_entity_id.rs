use crate::bootstrap::AppState;
use football_domain::{ExternalEntityIdDraft, ExternalEntityIdRecord};
use tauri::State;

#[tauri::command]
pub async fn add_external_entity_id(
    state: State<'_, AppState>,
    draft: ExternalEntityIdDraft,
) -> Result<ExternalEntityIdRecord, String> {
    state
        .service
        .add_external_entity_id(draft)
        .await
        .map_err(|error| error.to_string())
}
