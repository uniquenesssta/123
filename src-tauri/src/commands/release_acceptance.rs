use super::{parse_uuid, AppState};
use tauri::State;

#[tauri::command]
pub async fn run_release_acceptance(
    state: State<'_, AppState>,
    request: football_domain::ReleaseAcceptanceRequest,
) -> Result<football_domain::ReleaseAcceptanceRun, String> {
    state
        .service
        .run_release_acceptance(request)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn list_release_acceptance_runs(
    state: State<'_, AppState>,
    limit: u32,
) -> Result<Vec<football_domain::ReleaseAcceptanceRunSummary>, String> {
    state
        .service
        .list_release_acceptance_runs(limit)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn read_release_acceptance_run(
    state: State<'_, AppState>,
    run_id: String,
) -> Result<football_domain::ReleaseAcceptanceRun, String> {
    state
        .service
        .read_release_acceptance_run(parse_uuid(&run_id, "验收运行 ID")?)
        .await
        .map_err(|error| error.to_string())
}
