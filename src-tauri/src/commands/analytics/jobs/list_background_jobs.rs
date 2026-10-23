use crate::bootstrap::AppState;
use crate::commands::record_failed_background_jobs;
use tauri::State;

#[tauri::command]
pub async fn list_background_jobs(
    state: State<'_, AppState>,
    limit: u32,
) -> Result<Vec<football_domain::BackgroundJob>, String> {
    let jobs = state
        .service
        .list_background_jobs(limit)
        .await
        .map_err(|error| error.to_string())?;
    record_failed_background_jobs(&state.issue_log, &jobs);
    Ok(jobs)
}
