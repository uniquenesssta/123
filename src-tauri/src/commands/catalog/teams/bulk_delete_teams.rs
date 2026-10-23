use crate::{bootstrap::AppState, commands::parse_uuid};
use football_domain::BulkDeleteResult;
use tauri::State;

#[tauri::command]
pub async fn bulk_delete_teams(
    state: State<'_, AppState>,
    team_ids: Vec<String>,
) -> Result<BulkDeleteResult, String> {
    let ids = team_ids
        .iter()
        .map(|value| parse_uuid(value, "球队 ID"))
        .collect::<Result<Vec<_>, _>>()?;
    state
        .service
        .bulk_delete_teams(ids)
        .await
        .map_err(|error| error.to_string())
}
