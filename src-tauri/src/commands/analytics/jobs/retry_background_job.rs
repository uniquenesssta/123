use crate::{bootstrap::AppState, commands::parse_uuid};
use tauri::State;

#[tauri::command]
pub async fn retry_background_job(
    state: State<'_, AppState>,
    job_id: String,
) -> Result<football_domain::BackgroundJob, String> {
    let job_id = parse_uuid(&job_id, "任务 ID")?;
    state
        .service
        .retry_background_job(job_id)
        .await
        .map_err(|error| error.to_string())
}
