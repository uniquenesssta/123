use crate::{bootstrap::AppState, commands::parse_uuid};
use football_domain::TeamForceDeletePreview;
use tauri::State;

#[tauri::command]
pub async fn preview_force_delete_team(
    state: State<'_, AppState>,
    team_id: String,
) -> Result<TeamForceDeletePreview, String> {
    let team_id = parse_uuid(&team_id, "球队 ID")?;
    let service = state.service.clone();
    let runtime = tauri::async_runtime::handle();

    tauri::async_runtime::spawn_blocking(move || {
        runtime
            .block_on(service.preview_force_delete_team(team_id))
            .map_err(|error| error.to_string())
    })
    .await
    .map_err(|error| format!("球队强制删除预检任务执行失败：{error}"))?
}
