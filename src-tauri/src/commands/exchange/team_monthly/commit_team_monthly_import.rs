use crate::{bootstrap::AppState, commands::parse_uuid};
use tauri::State;

#[tauri::command]
pub async fn commit_team_monthly_import(
    state: State<'_, AppState>,
    batch_id: String,
) -> Result<football_domain::SpreadsheetImportCommitResult, String> {
    let batch_id = parse_uuid(&batch_id, "批次 ID")?;
    state
        .service
        .commit_team_monthly_import(batch_id)
        .await
        .map_err(|error| error.to_string())
}
