use super::mapping::job_from_row;
use crate::{PersistenceResult, PostgresStore};
use football_domain::BackgroundJob;
use sqlx::Row;
use uuid::Uuid;

impl PostgresStore {
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
}
