use super::mapping::job_from_row;
use crate::{PersistenceError, PersistenceResult, PostgresStore};
use football_domain::BackgroundJob;
use uuid::Uuid;

impl PostgresStore {
    pub async fn list_jobs(&self, limit: u32) -> PersistenceResult<Vec<BackgroundJob>> {
        let rows = sqlx::query(
            r#"
            SELECT id,job_type,status,progress,payload,result,error_message,priority,attempts,max_attempts,
                   cancellation_requested,available_at,created_at,started_at,finished_at,updated_at
            FROM platform.jobs ORDER BY created_at DESC,id DESC LIMIT $1
            "#,
        )
        .bind(i64::from(limit.clamp(1, 500)))
        .fetch_all(&self.pool)
        .await?;
        rows.iter().map(job_from_row).collect()
    }

    pub async fn read_job(&self, job_id: Uuid) -> PersistenceResult<BackgroundJob> {
        let row = sqlx::query(
            r#"SELECT id,job_type,status,progress,payload,result,error_message,priority,attempts,max_attempts,
                      cancellation_requested,available_at,created_at,started_at,finished_at,updated_at
               FROM platform.jobs WHERE id=$1"#,
        )
        .bind(job_id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| PersistenceError::InvalidState("任务不存在".to_string()))?;
        job_from_row(&row)
    }
}
