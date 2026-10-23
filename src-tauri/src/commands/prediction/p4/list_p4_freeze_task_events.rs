use crate::{bootstrap::AppState, commands::parse_uuid};
use football_domain::P4FreezeTaskEventRecord;
use tauri::State;

#[tauri::command]
pub async fn list_p4_freeze_task_events(
    state: State<'_, AppState>,
    task_id: String,
) -> Result<Vec<P4FreezeTaskEventRecord>, String> {
    state
        .service
        .list_p4_freeze_task_events(parse_uuid(&task_id, "冻结任务 ID")?)
        .await
        .map_err(|error| error.to_string())
}
