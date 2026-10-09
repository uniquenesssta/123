use super::events::append_task_event;
use super::input::{prepare, PreparedTask};
use super::row::{parse_state, task_from_row};
use crate::{write_audit_event, PersistenceError, PersistenceResult, PostgresStore};
use football_domain::{P4FreezeTaskDraft, P4FreezeTaskRecord, P4FreezeTaskTransition};
use serde_json::json;
use sqlx::{Postgres, Row, Transaction};
use uuid::Uuid;

impl PostgresStore {
    pub async fn create_p4_freeze_task(
        &self,
        draft: &P4FreezeTaskDraft,
    ) -> PersistenceResult<P4FreezeTaskRecord> {
        let PreparedTask {
            requested_fact_keys,
            task_fingerprint,
        } = prepare(draft)?;
        let mut tx = self.pool.begin().await?;
        lock_key(
            &mut tx,
            &format!("p4-freeze-task:{}", draft.idempotency_key),
        )
        .await?;
        if let Some(row) = select_task_by_idempotency(&mut tx, &draft.idempotency_key).await? {
            let existing: String = row.try_get("task_fingerprint")?;
            if existing != task_fingerprint {
                return Err(PersistenceError::InvalidState(format!(
                    "P4冻结任务幂等键 {} 已存在但载荷不同",
                    draft.idempotency_key
                )));
            }
            let record = task_from_row(&row)?;
            tx.commit().await?;
            return Ok(record);
        }

        let id = Uuid::new_v4();
        let row = sqlx::query(
            r#"
            INSERT INTO platform.p4_freeze_tasks (
                id, match_id, match_key, horizon, kickoff_at, data_cutoff_at,
                research_due_at, freeze_deadline_at, rule_package_id,
                model_version_id, parameter_set_id, competition_profile_id,
                research_schema_version_id, snapshot_schema_version_id,
                requested_fact_keys, trace_id, state, task_fingerprint,
                idempotency_key, metadata, updated_at
            ) VALUES (
                $1, $2, $3, $4, $5, $6,
                $7, $8, $9,
                $10, $11, $12,
                $13, $14,
                $15, $16, $17, $18,
                $19, $20, now()
            )
            RETURNING *
            "#,
        )
        .bind(id)
        .bind(draft.match_id)
        .bind(&draft.match_key)
        .bind(draft.horizon.as_str())
        .bind(draft.kickoff_at)
        .bind(draft.data_cutoff_at)
        .bind(draft.research_due_at)
        .bind(draft.freeze_deadline_at)
        .bind(draft.rule_package_id)
        .bind(draft.model_version_id)
        .bind(draft.parameter_set_id)
        .bind(draft.competition_profile_id)
        .bind(draft.research_schema_version_id)
        .bind(draft.snapshot_schema_version_id)
        .bind(&requested_fact_keys)
        .bind(draft.trace_id)
        .bind(draft.state.as_str())
        .bind(&task_fingerprint)
        .bind(&draft.idempotency_key)
        .bind(&draft.metadata)
        .fetch_one(&mut *tx)
        .await?;
        append_task_event(
            &mut tx,
            id,
            None,
            draft.state,
            "三个计划窗口规划器创建任务",
            json!({
                "planner_version": football_domain::P4_ORCHESTRATION_PLANNER_VERSION,
                "horizon": draft.horizon.as_str(),
                "data_cutoff_at": draft.data_cutoff_at,
            }),
        )
        .await?;
        write_audit_event(
            &mut tx,
            "p4_freeze_task_created",
            "p4_freeze_task",
            Some(id.to_string()),
            json!({
                "match_id": draft.match_id,
                "horizon": draft.horizon.as_str(),
                "data_cutoff_at": draft.data_cutoff_at,
                "state": draft.state.as_str(),
                "trace_id": draft.trace_id,
                "task_fingerprint": task_fingerprint,
            }),
        )
        .await?;
        let record = task_from_row(&row)?;
        tx.commit().await?;
        Ok(record)
    }

    pub async fn transition_p4_freeze_task(
        &self,
        transition: &P4FreezeTaskTransition,
    ) -> PersistenceResult<P4FreezeTaskRecord> {
        let mut tx = self.pool.begin().await?;
        let row = sqlx::query("SELECT * FROM platform.p4_freeze_tasks WHERE id = $1 FOR UPDATE")
            .bind(transition.task_id)
            .fetch_optional(&mut *tx)
            .await?
            .ok_or_else(|| PersistenceError::InvalidState("P4冻结任务不存在".to_string()))?;
        let current = parse_state(row.try_get::<String, _>("state")?.as_str())?;
        if current == transition.next_state {
            let record = task_from_row(&row)?;
            tx.commit().await?;
            return Ok(record);
        }
        if current != transition.expected_state {
            return Err(PersistenceError::InvalidState(format!(
                "P4冻结任务状态并发冲突：预期 {}，实际 {}",
                transition.expected_state.as_str(),
                current.as_str()
            )));
        }
        if !current.can_transition_to(transition.next_state) {
            return Err(PersistenceError::InvalidState(format!(
                "不允许的P4冻结状态迁移：{} -> {}",
                current.as_str(),
                transition.next_state.as_str()
            )));
        }

        sqlx::query(
            r#"
            UPDATE platform.p4_freeze_tasks
            SET state = $2,
                blockers = CASE WHEN $3 = 'null'::jsonb THEN blockers ELSE $3 END,
                research_run_id = COALESCE($4, research_run_id),
                research_job_id = COALESCE($5, research_job_id),
                freeze_job_id = COALESCE($6, freeze_job_id),
                snapshot_id = COALESCE($7, snapshot_id),
                updated_at = now()
            WHERE id = $1
            "#,
        )
        .bind(transition.task_id)
        .bind(transition.next_state.as_str())
        .bind(&transition.blockers)
        .bind(transition.research_run_id)
        .bind(transition.research_job_id)
        .bind(transition.freeze_job_id)
        .bind(transition.snapshot_id)
        .execute(&mut *tx)
        .await?;
        append_task_event(
            &mut tx,
            transition.task_id,
            Some(current),
            transition.next_state,
            &transition.reason,
            transition.payload.clone(),
        )
        .await?;
        write_audit_event(
            &mut tx,
            "p4_freeze_task_transitioned",
            "p4_freeze_task",
            Some(transition.task_id.to_string()),
            json!({
                "from_state": current.as_str(),
                "to_state": transition.next_state.as_str(),
                "reason": transition.reason,
            }),
        )
        .await?;
        let row = sqlx::query("SELECT * FROM platform.p4_freeze_tasks WHERE id = $1")
            .bind(transition.task_id)
            .fetch_one(&mut *tx)
            .await?;
        let record = task_from_row(&row)?;
        tx.commit().await?;
        Ok(record)
    }
}

async fn lock_key(tx: &mut Transaction<'_, Postgres>, key: &str) -> PersistenceResult<()> {
    sqlx::query("SELECT pg_advisory_xact_lock(hashtext($1)::bigint)")
        .bind(key)
        .execute(&mut **tx)
        .await?;
    Ok(())
}

async fn select_task_by_idempotency(
    tx: &mut Transaction<'_, Postgres>,
    idempotency_key: &str,
) -> PersistenceResult<Option<sqlx::postgres::PgRow>> {
    Ok(
        sqlx::query("SELECT * FROM platform.p4_freeze_tasks WHERE idempotency_key = $1")
            .bind(idempotency_key)
            .fetch_optional(&mut **tx)
            .await?,
    )
}
