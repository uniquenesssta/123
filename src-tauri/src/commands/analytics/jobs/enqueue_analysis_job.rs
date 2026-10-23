use crate::bootstrap::AppState;
use tauri::State;

#[tauri::command]
pub async fn enqueue_analysis_job(
    state: State<'_, AppState>,
    draft: football_domain::EnqueueJobDraft,
) -> Result<football_domain::BackgroundJob, String> {
    state
        .service
        .enqueue_analysis_job(draft)
        .await
        .map_err(|error| error.to_string())
}
