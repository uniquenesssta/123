use super::{sha256_json, write_audit_event, PersistenceError, PersistenceResult, PostgresStore};
use football_domain::{
    P4ManualConflictDecisionKind, P4ManualRouteOverrideDraft, P4ManualRouteOverrideRecord,
};
use serde_json::{json, Value};
use sqlx::{Postgres, Row, Transaction};
use uuid::Uuid;

impl PostgresStore {
    pub async fn append_p4_manual_route_override(
        &self,
        draft: &P4ManualRouteOverrideDraft,
    ) -> PersistenceResult<P4ManualRouteOverrideRecord> {
        let mut selected_evidence_ids = draft.selected_evidence_ids.clone();
        selected_evidence_ids.sort_unstable();
        selected_evidence_ids.dedup();
        let override_fingerprint = sha256_json(&json!({
            "task_id": draft.task_id,
            "research_run_id": draft.research_run_id,
            "conflict_id": draft.conflict_id,
            "route_key": draft.route_key,
            "field_key": draft.field_key,
            "target_module": draft.target_module,
            "target_slot": draft.target_slot,
            "entity_type": draft.entity_type,
            "entity_id": draft.entity_id,
            "decision_kind": draft.decision_kind.as_str(),
            "selected_evidence_ids": selected_evidence_ids,
            "selected_value": draft.selected_value,
            "verification_state": draft.verification_state,
            "route_status": draft.route_status,
            "reason": draft.reason,
            "actor": draft.actor,
            "note": draft.note,
        }))?;
        let mut tx = self.pool.begin().await?;
        lock_override(&mut tx, draft.task_id, &draft.route_key).await?;
        if let Some(row) = sqlx::query(
            r#"
            SELECT id, task_id, conflict_id, route_key, decision_kind,
                   selected_evidence_ids, route_status, verification_state,
                   override_fingerprint, created_at
            FROM research.manual_route_overrides
            WHERE idempotency_key = $1
            "#,
        )
        .bind(&draft.idempotency_key)
        .fetch_optional(&mut *tx)
        .await?
        {
            let existing: String = row.try_get("override_fingerprint")?;
            if existing != override_fingerprint {
                return Err(PersistenceError::InvalidState(
                    "人工冲突决策幂等键已存在但载荷不同".to_string(),
                ));
            }
            let record = manual_override_from_row(&row)?;
            tx.commit().await?;
            return Ok(record);
        }
        if let Some(existing) = sqlx::query_scalar::<_, String>(
            r#"
            SELECT override_fingerprint
            FROM research.manual_route_overrides
            WHERE task_id = $1 AND route_key = $2
            "#,
        )
        .bind(draft.task_id)
        .bind(&draft.route_key)
        .fetch_optional(&mut *tx)
        .await?
        {
            return Err(PersistenceError::InvalidState(format!(
                "该事实路由已经存在不可变人工决策，不能覆盖；既有指纹={existing}"
            )));
        }

        let route = sqlx::query(
            r#"
            SELECT id, field_key, target_module, target_slot, entity_type, entity_id,
                   route_status
            FROM research.evidence_routes
            WHERE research_run_id = $1 AND route_key = $2
            FOR SHARE
            "#,
        )
        .bind(draft.research_run_id)
        .bind(&draft.route_key)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| PersistenceError::InvalidState("需要处理的证据路由不存在".to_string()))?;
        if route.try_get::<String, _>("route_status")? != "blocked_conflict"
            || route.try_get::<String, _>("field_key")? != draft.field_key
            || route.try_get::<String, _>("target_module")? != draft.target_module
            || route.try_get::<String, _>("target_slot")? != draft.target_slot
            || route.try_get::<Option<String>, _>("entity_type")? != draft.entity_type
            || route.try_get::<Option<Uuid>, _>("entity_id")? != draft.entity_id
        {
            return Err(PersistenceError::InvalidState(
                "人工冲突决策与原始阻断路由不一致".to_string(),
            ));
        }
        let original_route_id: Uuid = route.try_get("id")?;

        let conflict = sqlx::query(
            r#"
            SELECT conflict.match_id, conflict.field_key, conflict.entity_type,
                   conflict.entity_id, task.match_id AS task_match_id,
                   task.research_run_id AS task_research_run_id
            FROM research.evidence_conflicts conflict
            JOIN platform.p4_freeze_tasks task ON task.id = $2
            WHERE conflict.id = $1
            FOR SHARE
            "#,
        )
        .bind(draft.conflict_id)
        .bind(draft.task_id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| PersistenceError::InvalidState("证据冲突不存在".to_string()))?;
        if conflict.try_get::<Uuid, _>("match_id")?
            != conflict.try_get::<Uuid, _>("task_match_id")?
            || conflict.try_get::<Option<Uuid>, _>("task_research_run_id")?
                != Some(draft.research_run_id)
            || conflict.try_get::<String, _>("field_key")? != draft.field_key
            || Some(conflict.try_get::<String, _>("entity_type")?) != draft.entity_type
            || conflict.try_get::<Option<Uuid>, _>("entity_id")? != draft.entity_id
        {
            return Err(PersistenceError::InvalidState(
                "证据冲突不属于当前任务或原始路由".to_string(),
            ));
        }

        validate_selected_conflict_evidence(&mut tx, draft, &selected_evidence_ids).await?;

        let id = Uuid::new_v4();
        let row = sqlx::query(
            r#"
            INSERT INTO research.manual_route_overrides (
                id, task_id, research_run_id, conflict_id, original_route_id,
                route_key, field_key, target_module, target_slot, entity_type,
                entity_id, decision_kind, selected_evidence_ids, selected_value,
                route_status, verification_state, reason, actor, note,
                idempotency_key, override_fingerprint
            ) VALUES (
                $1, $2, $3, $4, $5,
                $6, $7, $8, $9, $10,
                $11, $12, $13, $14,
                $15, $16, $17, $18, $19,
                $20, $21
            )
            RETURNING id, task_id, conflict_id, route_key, decision_kind,
                      selected_evidence_ids, route_status, verification_state,
                      override_fingerprint, created_at
            "#,
        )
        .bind(id)
        .bind(draft.task_id)
        .bind(draft.research_run_id)
        .bind(draft.conflict_id)
        .bind(original_route_id)
        .bind(&draft.route_key)
        .bind(&draft.field_key)
        .bind(&draft.target_module)
        .bind(&draft.target_slot)
        .bind(&draft.entity_type)
        .bind(draft.entity_id)
        .bind(draft.decision_kind.as_str())
        .bind(&selected_evidence_ids)
        .bind(&draft.selected_value)
        .bind(&draft.route_status)
        .bind(&draft.verification_state)
        .bind(&draft.reason)
        .bind(&draft.actor)
        .bind(&draft.note)
        .bind(&draft.idempotency_key)
        .bind(&override_fingerprint)
        .fetch_one(&mut *tx)
        .await?;

        let event_type = match draft.decision_kind {
            P4ManualConflictDecisionKind::SelectEvidence => "resolved",
            P4ManualConflictDecisionKind::AcceptUnknown => "accepted_unknown",
        };
        append_conflict_event_in_tx(
            &mut tx,
            draft.conflict_id,
            event_type,
            &draft.actor,
            json!({
                "task_id": draft.task_id,
                "route_key": draft.route_key,
                "decision_kind": draft.decision_kind.as_str(),
                "selected_evidence_ids": selected_evidence_ids,
                "note": draft.note,
                "override_id": id,
            }),
            &draft.idempotency_key,
        )
        .await?;
        write_audit_event(
            &mut tx,
            "p4_manual_conflict_resolved",
            "evidence_conflict",
            Some(draft.conflict_id.to_string()),
            json!({
                "task_id": draft.task_id,
                "route_key": draft.route_key,
                "decision_kind": draft.decision_kind.as_str(),
                "override_id": id,
            }),
        )
        .await?;
        let record = manual_override_from_row(&row)?;
        tx.commit().await?;
        Ok(record)
    }
}

async fn validate_selected_conflict_evidence(
    tx: &mut Transaction<'_, Postgres>,
    draft: &P4ManualRouteOverrideDraft,
    selected_evidence_ids: &[Uuid],
) -> PersistenceResult<()> {
    match draft.decision_kind {
        P4ManualConflictDecisionKind::SelectEvidence => {
            if selected_evidence_ids.is_empty()
                || draft.route_status != "routed"
                || draft.verification_state != "PROBABLE"
            {
                return Err(PersistenceError::InvalidState(
                    "选择证据的人工决策必须产生PROBABLE routed路由".to_string(),
                ));
            }
            let rows = sqlx::query(
                r#"
                SELECT claim.id, claim.value
                FROM research.evidence_claims claim
                JOIN research.evidence_conflict_members member ON member.evidence_id = claim.id
                WHERE member.conflict_id = $1
                  AND claim.id = ANY($2)
                  AND claim.research_run_id = $3
                  AND claim.field_key = $4
                  AND claim.entity_type = $5
                  AND claim.entity_id IS NOT DISTINCT FROM $6
                ORDER BY claim.id
                "#,
            )
            .bind(draft.conflict_id)
            .bind(selected_evidence_ids)
            .bind(draft.research_run_id)
            .bind(&draft.field_key)
            .bind(&draft.entity_type)
            .bind(draft.entity_id)
            .fetch_all(&mut **tx)
            .await?;
            if rows.len() != selected_evidence_ids.len() {
                return Err(PersistenceError::InvalidState(
                    "所选证据不完全属于当前冲突组".to_string(),
                ));
            }
            let values = rows
                .iter()
                .map(|row| row.try_get::<Value, _>("value"))
                .collect::<Result<Vec<_>, _>>()?;
            let Some(first) = values.first() else {
                return Err(PersistenceError::InvalidState(
                    "人工冲突决策缺少证据值".to_string(),
                ));
            };
            if values.iter().any(|value| value != first) || &draft.selected_value != first {
                return Err(PersistenceError::InvalidState(
                    "一次人工决策只能选择值完全一致的证据".to_string(),
                ));
            }
        }
        P4ManualConflictDecisionKind::AcceptUnknown => {
            if !selected_evidence_ids.is_empty()
                || draft.route_status != "missing"
                || draft.verification_state != "NOT_FOUND"
                || !draft.selected_value.is_null()
            {
                return Err(PersistenceError::InvalidState(
                    "接受未知必须产生NOT_FOUND missing路由且不得选择证据".to_string(),
                ));
            }
        }
    }
    Ok(())
}

async fn append_conflict_event_in_tx(
    tx: &mut Transaction<'_, Postgres>,
    conflict_id: Uuid,
    event_type: &str,
    actor: &str,
    payload: Value,
    decision_key: &str,
) -> PersistenceResult<()> {
    let idempotency_key = format!("manual:{decision_key}");
    let event_fingerprint = sha256_json(&json!({
        "event_type": event_type,
        "actor": actor,
        "payload": payload,
    }))?;
    let inserted: Option<Uuid> = sqlx::query_scalar(
        r#"
        INSERT INTO research.evidence_conflict_events (
            id, conflict_id, event_type, actor, payload,
            idempotency_key, event_fingerprint
        ) VALUES ($1, $2, $3, $4, $5, $6, $7)
        ON CONFLICT (conflict_id, idempotency_key) DO NOTHING
        RETURNING id
        "#,
    )
    .bind(Uuid::new_v4())
    .bind(conflict_id)
    .bind(event_type)
    .bind(actor)
    .bind(&payload)
    .bind(&idempotency_key)
    .bind(&event_fingerprint)
    .fetch_optional(&mut **tx)
    .await?;
    if inserted.is_none() {
        let existing: String = sqlx::query_scalar(
            r#"
            SELECT event_fingerprint
            FROM research.evidence_conflict_events
            WHERE conflict_id = $1 AND idempotency_key = $2
            "#,
        )
        .bind(conflict_id)
        .bind(&idempotency_key)
        .fetch_one(&mut **tx)
        .await?;
        if existing != event_fingerprint {
            return Err(PersistenceError::InvalidState(
                "人工冲突事件幂等冲突".to_string(),
            ));
        }
    }
    Ok(())
}

async fn lock_override(
    tx: &mut Transaction<'_, Postgres>,
    task_id: Uuid,
    route_key: &str,
) -> PersistenceResult<()> {
    sqlx::query("SELECT pg_advisory_xact_lock(hashtext($1)::bigint)")
        .bind(format!("p4-manual-route:{task_id}:{route_key}"))
        .execute(&mut **tx)
        .await?;
    Ok(())
}

fn manual_override_from_row(
    row: &sqlx::postgres::PgRow,
) -> PersistenceResult<P4ManualRouteOverrideRecord> {
    let decision_kind = match row.try_get::<String, _>("decision_kind")?.as_str() {
        "select_evidence" => P4ManualConflictDecisionKind::SelectEvidence,
        "accept_unknown" => P4ManualConflictDecisionKind::AcceptUnknown,
        other => {
            return Err(PersistenceError::InvalidState(format!(
                "未知人工冲突决策：{other}"
            )))
        }
    };
    Ok(P4ManualRouteOverrideRecord {
        id: row.try_get("id")?,
        task_id: row.try_get("task_id")?,
        conflict_id: row.try_get("conflict_id")?,
        route_key: row.try_get("route_key")?,
        decision_kind,
        selected_evidence_ids: row.try_get("selected_evidence_ids")?,
        route_status: row.try_get("route_status")?,
        verification_state: row.try_get("verification_state")?,
        created_at: row.try_get("created_at")?,
    })
}
