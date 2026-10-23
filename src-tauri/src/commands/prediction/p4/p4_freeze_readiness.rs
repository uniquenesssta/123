use crate::{bootstrap::AppState, commands::parse_uuid};
use football_domain::P4FreezeReadiness;
use tauri::State;

#[tauri::command]
pub async fn p4_freeze_readiness(
    state: State<'_, AppState>,
    task_id: String,
) -> Result<P4FreezeReadiness, String> {
    state
        .service
        .p4_freeze_readiness(parse_uuid(&task_id, "冻结任务 ID")?)
        .await
        .map_err(|error| error.to_string())
}
