use super::policy::job_status;
use crate::PersistenceResult;
use football_domain::BackgroundJob;
use sqlx::Row;

pub(super) fn job_from_row(row: &sqlx::postgres::PgRow) -> PersistenceResult<BackgroundJob> {
    let status: String = row.try_get("status")?;
    let status = job_status(&status)?;
    Ok(BackgroundJob {
        id: row.try_get("id")?,
        job_type: row.try_get("job_type")?,
        status,
        progress: row.try_get("progress")?,
        payload: row.try_get("payload")?,
        result: row.try_get("result")?,
        error_message: row.try_get("error_message")?,
        priority: row.try_get("priority")?,
        attempts: row.try_get("attempts")?,
        max_attempts: row.try_get("max_attempts")?,
        cancellation_requested: row.try_get("cancellation_requested")?,
        available_at: row.try_get("available_at")?,
        created_at: row.try_get("created_at")?,
        started_at: row.try_get("started_at")?,
        finished_at: row.try_get("finished_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}
