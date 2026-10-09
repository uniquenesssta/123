use super::row::{task_event_from_row, task_from_row};
use crate::{PersistenceError, PersistenceResult, PostgresStore};
use football_domain::{P4FreezeTaskEventRecord, P4FreezeTaskRecord};
use uuid::Uuid;

impl PostgresStore {
    pub async fn find_p4_freeze_task_by_idempotency(
        &self,
        idempotency_key: &str,
    ) -> PersistenceResult<Option<P4FreezeTaskRecord>> {
        let row = sqlx::query("SELECT * FROM platform.p4_freeze_tasks WHERE idempotency_key = $1")
            .bind(idempotency_key)
            .fetch_optional(&self.pool)
            .await?;
        row.as_ref().map(task_from_row).transpose()
    }

    pub async fn read_p4_freeze_task(
        &self,
        task_id: Uuid,
    ) -> PersistenceResult<P4FreezeTaskRecord> {
        let row = sqlx::query("SELECT * FROM platform.p4_freeze_tasks WHERE id = $1")
            .bind(task_id)
            .fetch_optional(&self.pool)
            .await?
            .ok_or_else(|| PersistenceError::InvalidState("P4冻结任务不存在".to_string()))?;
        task_from_row(&row)
    }

    pub async fn list_p4_freeze_tasks(
        &self,
        match_id: Option<Uuid>,
        limit: u32,
    ) -> PersistenceResult<Vec<P4FreezeTaskRecord>> {
        let rows = sqlx::query(
            r#"
            SELECT *
            FROM platform.p4_freeze_tasks
            WHERE $1::uuid IS NULL OR match_id = $1
            ORDER BY kickoff_at DESC,
                     CASE horizon
                       WHEN 'T-24h' THEN 1
                       WHEN 'T-6h' THEN 2
                       WHEN 'T-90m' THEN 3
                       WHEN 'T-1h' THEN 4
                       ELSE 5
                     END,
                     created_at DESC
            LIMIT $2
            "#,
        )
        .bind(match_id)
        .bind(i64::from(limit.clamp(1, 500)))
        .fetch_all(&self.pool)
        .await?;
        rows.iter().map(task_from_row).collect()
    }

    pub async fn list_p4_freeze_task_events(
        &self,
        task_id: Uuid,
    ) -> PersistenceResult<Vec<P4FreezeTaskEventRecord>> {
        let rows = sqlx::query(
            r#"
            SELECT id, task_id, from_state, to_state, reason, payload, occurred_at
            FROM platform.p4_freeze_task_events
            WHERE task_id = $1
            ORDER BY occurred_at, id
            "#,
        )
        .bind(task_id)
        .fetch_all(&self.pool)
        .await?;
        rows.iter().map(task_event_from_row).collect()
    }
}
