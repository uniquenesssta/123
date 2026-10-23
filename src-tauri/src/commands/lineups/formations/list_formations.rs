use crate::bootstrap::AppState;
use football_domain::FormationRecord;
use tauri::State;

#[tauri::command]
pub async fn list_formations(
    state: State<'_, AppState>,
    active_only: bool,
) -> Result<Vec<FormationRecord>, String> {
    state
        .service
        .list_formations(active_only)
        .await
        .map_err(|error| error.to_string())
}
