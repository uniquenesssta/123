use crate::{bootstrap::AppState, commands::parse_uuid};
use football_domain::P4FreezeTaskRecord;
use tauri::State;

#[tauri::command]
pub async fn list_p4_freeze_tasks(
    state: State<'_, AppState>,
    match_id: Option<String>,
    limit: u32,
) -> Result<Vec<P4FreezeTaskRecord>, String> {
    let match_id = match_id
        .as_deref()
        .map(|value| parse_uuid(value, "比赛 ID"))
        .transpose()?;
    state
        .service
        .list_p4_freeze_tasks(match_id, limit)
        .await
        .map_err(|error| error.to_string())
}
