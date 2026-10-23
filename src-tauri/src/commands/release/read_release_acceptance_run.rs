use crate::bootstrap::AppState;
use crate::commands::parse_uuid;
use tauri::State;

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
