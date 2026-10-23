use crate::bootstrap::AppState;
use football_domain::{P4FreezeTaskRecord, PlanP4HorizonsCommand};
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
