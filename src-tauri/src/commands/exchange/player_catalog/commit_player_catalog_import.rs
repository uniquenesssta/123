use crate::{bootstrap::AppState, commands::parse_uuid};
use tauri::State;

#[tauri::command]
pub async fn commit_player_catalog_import(
    state: State<'_, AppState>,
    batch_id: String,
) -> Result<football_domain::SpreadsheetImportCommitResult, String> {
    let parsed = parse_uuid(&batch_id, "批次 ID")?;
    state
        .service
        .commit_player_catalog_import(parsed)
        .await
        .map_err(|error| error.to_string())
}
