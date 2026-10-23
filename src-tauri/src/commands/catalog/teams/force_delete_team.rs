use crate::bootstrap::AppState;
use football_domain::{TeamForceDeleteRequest, TeamForceDeleteResult};
use tauri::State;

#[tauri::command]
pub async fn force_delete_team(
    state: State<'_, AppState>,
    request: TeamForceDeleteRequest,
) -> Result<TeamForceDeleteResult, String> {
    let service = state.service.clone();
    let runtime = tauri::async_runtime::handle();

    tauri::async_runtime::spawn_blocking(move || {
        runtime
            .block_on(service.force_delete_team(request))
            .map_err(|error| error.to_string())
    })
    .await
    .map_err(|error| format!("球队强制删除任务执行失败：{error}"))?
}
