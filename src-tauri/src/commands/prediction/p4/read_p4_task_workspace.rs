use crate::{bootstrap::AppState, commands::parse_uuid};
use football_domain::P4TaskWorkspace;
use tauri::State;

#[tauri::command]
pub async fn read_p4_task_workspace(
    state: State<'_, AppState>,
    task_id: String,
) -> Result<P4TaskWorkspace, String> {
    state
        .service
        .read_p4_task_workspace(parse_uuid(&task_id, "冻结任务 ID")?)
        .await
        .map_err(|error| error.to_string())
}
