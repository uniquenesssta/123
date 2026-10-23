use crate::{bootstrap::AppState, commands::parse_uuid};
use football_domain::BulkDeleteResult;
use tauri::State;

#[tauri::command]
pub async fn bulk_delete_players(
    state: State<'_, AppState>,
    player_ids: Vec<String>,
) -> Result<BulkDeleteResult, String> {
    let ids = player_ids
        .iter()
        .map(|value| parse_uuid(value, "球员 ID"))
        .collect::<Result<Vec<_>, _>>()?;
    state
        .service
        .bulk_delete_players(ids)
        .await
        .map_err(|error| error.to_string())
}
