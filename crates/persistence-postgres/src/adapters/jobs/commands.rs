use super::{mapping::job_from_row, policy::validate_job_type};
use crate::{PersistenceError, PersistenceResult, PostgresStore};
use football_domain::{BackgroundJob, EnqueueJobDraft};
use uuid::Uuid;

impl PostgresStore {
    pub async fn enqueue_job(&self, draft: &EnqueueJobDraft) -> PersistenceResult<BackgroundJob> {
        validate_job_type(&draft.job_type)?;
        let id = Uuid::new_v4();
        let row = sqlx::query(
            r#"
            INSERT INTO platform.jobs (
                id,job_type,status,progress,payload,idempotency_key,available_at,
                priority,max_attempts,updated_at
            ) VALUES ($1,$2,'queued',0,$3,$4,COALESCE($5,now()),$6,$7,now())
            ON CONFLICT (idempotency_key) DO UPDATE SET idempotency_key=EXCLUDED.idempotency_key
            RETURNING id,job_type,status,progress,payload,result,error_message,priority,attempts,max_attempts,
                      cancellation_requested,available_at,created_at,started_at,finished_at,updated_at
            "#,
        )
        .bind(id)
        .bind(&draft.job_type)
        .bind(&draft.payload)
        .bind(&draft.idempotency_key)
        .bind(draft.available_at)
        .bind(draft.priority)
        .bind(draft.max_attempts.clamp(1, 20))
        .fetch_one(&self.pool)
        .await?;
        job_from_row(&row)
    }

    pub async fn request_job_cancellation(&self, job_id: Uuid) -> PersistenceResult<BackgroundJob> {
        sqlx::query(
            r#"
            UPDATE platform.jobs
            SET cancellation_requested=true,
                status=CASE WHEN status='queued' THEN 'cancelled' ELSE status END,
                finished_at=CASE WHEN status='queued' THEN now() ELSE finished_at END,
                updated_at=now()
            WHERE id=$1 AND status IN ('queued','running')
            "#,
        )
        .bind(job_id)
        .execute(&self.pool)
        .await?;
        self.read_job(job_id).await
    }

    pub async fn retry_job(&self, job_id: Uuid) -> PersistenceResult<BackgroundJob> {
        let result = sqlx::query(
            r#"
            UPDATE platform.jobs
            SET status='queued',progress=0,result=NULL,error_message=NULL,cancellation_requested=false,
                available_at=now(),started_at=NULL,finished_at=NULL,heartbeat_at=NULL,updated_at=now()
            WHERE id=$1 AND status IN ('failed','cancelled') AND attempts < max_attempts
            "#,
        )
        .bind(job_id)
        .execute(&self.pool)
        .await?;
        if result.rows_affected() == 0 {
            return Err(PersistenceError::InvalidState(
                "任务不可重试，可能仍在运行或已达到最大尝试次数".to_string(),
            ));
        }
        self.read_job(job_id).await
    }
}
