use crate::bootstrap::AppState;
use tauri::State;

#[tauri::command]
pub async fn postmatch_overview(
    state: State<'_, AppState>,
    limit: u32,
) -> Result<football_domain::PostmatchOverview, String> {
    state
        .service
        .postmatch_overview(limit)
        .await
        .map_err(|error| error.to_string())
}
