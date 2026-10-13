use super::policy::failure_disposition;
use crate::{PersistenceError, PersistenceResult, PostgresStore};
use serde_json::{json, Value};
use sqlx::Row;
use uuid::Uuid;

impl PostgresStore {
    pub async fn recover_interrupted_jobs(&self) -> PersistenceResult<u64> {
        let result = sqlx::query(
            r#"
            UPDATE platform.jobs
            SET status=CASE WHEN attempts < max_attempts THEN 'queued' ELSE 'failed' END,
                available_at=CASE WHEN attempts < max_attempts THEN now() ELSE available_at END,
                started_at=NULL,
                heartbeat_at=NULL,
                finished_at=CASE WHEN attempts < max_attempts THEN NULL ELSE now() END,
                error_message=COALESCE(error_message,'') || CASE WHEN COALESCE(error_message,'')='' THEN '' ELSE E'\n' END ||
                    CASE WHEN attempts < max_attempts THEN '应用重启后重新排队' ELSE '应用重启时已达到最大尝试次数' END,
                updated_at=now()
            WHERE status='running'
            "#,
        )
        .execute(&self.pool)
        .await?;
        Ok(result.rows_affected())
    }

    pub async fn update_job_progress(
        &self,
        job_id: Uuid,
        progress: f64,
        message: &str,
        payload: Value,
    ) -> PersistenceResult<bool> {
        let mut tx = self.pool.begin().await?;
        let row = sqlx::query(
            "UPDATE platform.jobs SET progress=$2,heartbeat_at=now(),updated_at=now() WHERE id=$1 AND status='running' RETURNING cancellation_requested",
        )
        .bind(job_id)
        .bind(progress.clamp(0.0, 99.0))
        .fetch_optional(&mut *tx)
        .await?;
        let Some(row) = row else {
            tx.commit().await?;
            return Ok(true);
        };
        let cancellation_requested: bool = row.try_get("cancellation_requested")?;
        sqlx::query("INSERT INTO platform.job_events (id,job_id,event_type,progress,message,payload) VALUES ($1,$2,'progress',$3,$4,$5)")
            .bind(Uuid::new_v4())
            .bind(job_id)
            .bind(progress.clamp(0.0, 99.0))
            .bind(message)
            .bind(payload)
            .execute(&mut *tx)
            .await?;
        tx.commit().await?;
        Ok(cancellation_requested)
    }

    pub async fn complete_job(&self, job_id: Uuid, result: Value) -> PersistenceResult<()> {
        let mut tx = self.pool.begin().await?;
        let row = sqlx::query(
            r#"
            UPDATE platform.jobs
            SET status=CASE WHEN cancellation_requested THEN 'cancelled' ELSE 'succeeded' END,
                progress=CASE WHEN cancellation_requested THEN progress ELSE 100 END,
                result=CASE WHEN cancellation_requested THEN NULL ELSE $2 END,
                finished_at=now(),heartbeat_at=now(),updated_at=now()
            WHERE id=$1 AND status='running'
            RETURNING status,progress
            "#,
        )
        .bind(job_id)
        .bind(&result)
        .fetch_optional(&mut *tx)
        .await?;
        let Some(row) = row else {
            tx.commit().await?;
            return Ok(());
        };
        let status: String = row.try_get("status")?;
        let progress: f64 = row.try_get("progress")?;
        let message = if status == "cancelled" {
            "任务已取消"
        } else {
            "任务已完成"
        };
        let event_payload = if status == "cancelled" {
            Value::Null
        } else {
            result
        };
        sqlx::query("INSERT INTO platform.job_events (id,job_id,event_type,progress,message,payload) VALUES ($1,$2,$3,$4,$5,$6)")
            .bind(Uuid::new_v4())
            .bind(job_id)
            .bind(&status)
            .bind(progress)
            .bind(message)
            .bind(event_payload)
            .execute(&mut *tx)
            .await?;
        tx.commit().await?;
        Ok(())
    }

    pub async fn fail_job(&self, job_id: Uuid, error_message: &str) -> PersistenceResult<()> {
        let mut tx = self.pool.begin().await?;
        let row = sqlx::query("SELECT attempts,max_attempts,cancellation_requested FROM platform.jobs WHERE id=$1 FOR UPDATE")
            .bind(job_id).fetch_optional(&mut *tx).await?
            .ok_or_else(|| PersistenceError::InvalidState("任务不存在".to_string()))?;
        let attempts: i32 = row.try_get("attempts")?;
        let max_attempts: i32 = row.try_get("max_attempts")?;
        let cancellation_requested: bool = row.try_get("cancellation_requested")?;
        let (status, finished) =
            failure_disposition(attempts, max_attempts, cancellation_requested);
        sqlx::query("UPDATE platform.jobs SET status=$2,error_message=$3,progress=CASE WHEN $2='queued' THEN 0 ELSE progress END,available_at=CASE WHEN $2='queued' THEN now() + interval '30 seconds' ELSE available_at END,finished_at=CASE WHEN $4 THEN now() ELSE NULL END,heartbeat_at=NULL,updated_at=now() WHERE id=$1")
            .bind(job_id).bind(status).bind(error_message).bind(finished).execute(&mut *tx).await?;
        sqlx::query("INSERT INTO platform.job_events (id,job_id,event_type,progress,message,payload) VALUES ($1,$2,$3,NULL,$4,$5)")
            .bind(Uuid::new_v4()).bind(job_id).bind(status).bind(error_message)
            .bind(json!({"attempts":attempts,"max_attempts":max_attempts})).execute(&mut *tx).await?;
        tx.commit().await?;
        Ok(())
    }
}
