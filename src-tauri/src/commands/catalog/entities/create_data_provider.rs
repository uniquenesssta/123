use crate::bootstrap::AppState;
use football_domain::{DataProviderDraft, DataProviderRecord};
use tauri::State;

#[tauri::command]
pub async fn create_data_provider(
    state: State<'_, AppState>,
    draft: DataProviderDraft,
) -> Result<DataProviderRecord, String> {
    state
        .service
        .create_data_provider(draft)
        .await
        .map_err(|error| error.to_string())
}
