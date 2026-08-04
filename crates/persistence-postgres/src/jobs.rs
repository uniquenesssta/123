use super::{PersistenceError, PersistenceResult, PostgresStore};
use football_domain::{BackgroundJob, EnqueueJobDraft, JobStatus};
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

    pub async fn claim_next_job(&self) -> PersistenceResult<Option<BackgroundJob>> {
        self.claim_next_job_by_types(&[
            "refresh_analytics",
            "data_quality_scan",
            "query_performance_scan",
            "full_analysis_refresh",
            "p4_horizon_research",
            "p4_horizon_freeze",
        ])
        .await
    }

    pub async fn claim_next_job_by_types(
        &self,
        job_types: &[&str],
    ) -> PersistenceResult<Option<BackgroundJob>> {
        if job_types.is_empty() {
            return Ok(None);
        }
        let job_types = job_types
            .iter()
            .map(|value| (*value).to_string())
            .collect::<Vec<_>>();
        let mut tx = self.pool.begin().await?;
        let row = sqlx::query(
            r#"
            SELECT id
            FROM platform.jobs
            WHERE status='queued'
              AND cancellation_requested=false
              AND attempts < max_attempts
              AND available_at <= now()
              AND job_type = ANY($1)
            ORDER BY priority DESC,available_at,created_at,id
            FOR UPDATE SKIP LOCKED
            LIMIT 1
            "#,
        )
        .bind(&job_types)
        .fetch_optional(&mut *tx)
        .await?;
        let Some(row) = row else {
            tx.commit().await?;
            return Ok(None);
        };
        let id: Uuid = row.try_get("id")?;
        let claimed = sqlx::query(
            r#"
            UPDATE platform.jobs
            SET status='running',progress=1,attempts=attempts+1,started_at=COALESCE(started_at,now()),
                heartbeat_at=now(),updated_at=now(),error_message=NULL
            WHERE id=$1
            RETURNING id,job_type,status,progress,payload,result,error_message,priority,attempts,max_attempts,
                      cancellation_requested,available_at,created_at,started_at,finished_at,updated_at
            "#,
        )
        .bind(id)
        .fetch_one(&mut *tx)
        .await?;
        sqlx::query("INSERT INTO platform.job_events (id,job_id,event_type,progress,message) VALUES ($1,$2,'started',1,'任务已开始')")
            .bind(Uuid::new_v4())
            .bind(id)
            .execute(&mut *tx)
            .await?;
        tx.commit().await?;
        Ok(Some(job_from_row(&claimed)?))
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
        let (status, finished) = if cancellation_requested {
            ("cancelled", true)
        } else if attempts < max_attempts {
            ("queued", false)
        } else {
            ("failed", true)
        };
        sqlx::query("UPDATE platform.jobs SET status=$2,error_message=$3,progress=CASE WHEN $2='queued' THEN 0 ELSE progress END,available_at=CASE WHEN $2='queued' THEN now() + interval '30 seconds' ELSE available_at END,finished_at=CASE WHEN $4 THEN now() ELSE NULL END,heartbeat_at=NULL,updated_at=now() WHERE id=$1")
            .bind(job_id).bind(status).bind(error_message).bind(finished).execute(&mut *tx).await?;
        sqlx::query("INSERT INTO platform.job_events (id,job_id,event_type,progress,message,payload) VALUES ($1,$2,$3,NULL,$4,$5)")
            .bind(Uuid::new_v4()).bind(job_id).bind(status).bind(error_message)
            .bind(json!({"attempts":attempts,"max_attempts":max_attempts})).execute(&mut *tx).await?;
        tx.commit().await?;
        Ok(())
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

fn validate_job_type(job_type: &str) -> PersistenceResult<()> {
    if matches!(
        job_type,
        "refresh_analytics"
            | "data_quality_scan"
            | "query_performance_scan"
            | "full_analysis_refresh"
            | "p4_horizon_research"
            | "p4_horizon_freeze"
    ) {
        Ok(())
    } else {
        Err(PersistenceError::InvalidState(format!(
            "不支持的任务类型：{job_type}"
        )))
    }
}

fn job_from_row(row: &sqlx::postgres::PgRow) -> PersistenceResult<BackgroundJob> {
    let status: String = row.try_get("status")?;
    let status = match status.as_str() {
        "queued" => JobStatus::Queued,
        "running" => JobStatus::Running,
        "succeeded" => JobStatus::Succeeded,
        "failed" => JobStatus::Failed,
        "cancelled" => JobStatus::Cancelled,
        other => {
            return Err(PersistenceError::InvalidState(format!(
                "未知任务状态：{other}"
            )))
        }
    };
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
