use super::{parse_uuid, AppState};
use football_domain::{
    P4FreezeReadiness, P4FreezeTaskEventRecord, P4FreezeTaskRecord, P4MatchWorkspace,
    P4TaskWorkspace, PlanP4HorizonsCommand, ResolveP4ConflictCommand,
};
use tauri::State;

#[tauri::command]
pub async fn plan_p4_horizons(
    state: State<'_, AppState>,
    command: PlanP4HorizonsCommand,
) -> Result<Vec<P4FreezeTaskRecord>, String> {
    state
        .service
        .plan_p4_horizons(command)
        .await
        .map_err(|error| error.to_string())
}

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

#[tauri::command]
pub async fn read_p4_freeze_task(
    state: State<'_, AppState>,
    task_id: String,
) -> Result<P4FreezeTaskRecord, String> {
    state
        .service
        .read_p4_freeze_task(parse_uuid(&task_id, "冻结任务 ID")?)
        .await
        .map_err(|error| error.to_string())
}

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

#[tauri::command]
pub async fn read_p4_match_workspace(
    state: State<'_, AppState>,
    match_id: String,
) -> Result<P4MatchWorkspace, String> {
    state
        .service
        .read_p4_match_workspace(parse_uuid(&match_id, "比赛 ID")?)
        .await
        .map_err(|error| error.to_string())
}

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

#[tauri::command]
pub async fn resolve_p4_conflict(
    state: State<'_, AppState>,
    command: ResolveP4ConflictCommand,
) -> Result<P4TaskWorkspace, String> {
    state
        .service
        .resolve_p4_conflict(command)
        .await
        .map_err(|error| error.to_string())
}
