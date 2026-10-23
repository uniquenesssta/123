use crate::bootstrap::AppState;
use tauri::State;

#[tauri::command]
pub async fn list_reviewable_matches(
    state: State<'_, AppState>,
    limit: u32,
) -> Result<Vec<football_domain::ReviewableMatch>, String> {
    state
        .service
        .list_reviewable_matches(limit)
        .await
        .map_err(|error| error.to_string())
}
