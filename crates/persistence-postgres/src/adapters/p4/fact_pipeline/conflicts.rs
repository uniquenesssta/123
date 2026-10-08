use super::fingerprint::ensure_fingerprint;
use crate::{sha256_json, PersistenceError, PersistenceResult, PostgresStore};
use football_domain::{
    ConflictEvaluationDraft, ConflictEvaluationRecord, ConflictEvaluationStatus,
};
use serde_json::{json, Value};
use sqlx::Row;
use uuid::Uuid;

impl PostgresStore {
    pub async fn append_conflict_evaluation(
        &self,
        draft: &ConflictEvaluationDraft,
    ) -> PersistenceResult<ConflictEvaluationRecord> {
        let fingerprint = sha256_json(&json!({
            "conflict_id": draft.conflict_id,
            "research_run_id": draft.research_run_id,
            "match_id": draft.match_id,
            "trace_id": draft.trace_id,
            "source_policy_key": draft.source_policy_key,
            "source_policy_version": draft.source_policy_version,
            "status": draft.status.as_str(),
            "winning_evidence_ids": draft.winning_evidence_ids,
            "winning_value": draft.winning_value,
            "ranking": draft.ranking,
            "reason": draft.reason,
        }))?;
        let row = sqlx::query(
            r#"
            INSERT INTO research.conflict_evaluations (
                id, conflict_id, research_run_id, match_id, trace_id,
                source_policy_key, source_policy_version, evaluation_status,
                winning_evidence_ids, winning_value, ranking, reason,
                idempotency_key, evaluation_fingerprint
            ) VALUES (
                $1, $2, $3, $4, $5,
                $6, $7, $8, $9, $10,
                $11, $12, $13, $14
            )
            ON CONFLICT (idempotency_key) DO NOTHING
            RETURNING id, evaluation_status, evaluation_fingerprint, created_at
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(draft.conflict_id)
        .bind(draft.research_run_id)
        .bind(draft.match_id)
        .bind(draft.trace_id)
        .bind(&draft.source_policy_key)
        .bind(&draft.source_policy_version)
        .bind(draft.status.as_str())
        .bind(&draft.winning_evidence_ids)
        .bind(&draft.winning_value)
        .bind(&draft.ranking)
        .bind(&draft.reason)
        .bind(&draft.idempotency_key)
        .bind(&fingerprint)
        .fetch_optional(&self.pool)
        .await?;
        let row = match row {
            Some(row) => row,
            None => {
                sqlx::query(
                    r#"SELECT id, evaluation_status, evaluation_fingerprint, created_at
                   FROM research.conflict_evaluations
                   WHERE idempotency_key = $1"#,
                )
                .bind(&draft.idempotency_key)
                .fetch_one(&self.pool)
                .await?
            }
        };
        let existing: String = row.try_get("evaluation_fingerprint")?;
        ensure_fingerprint("冲突评估", &draft.idempotency_key, &existing, &fingerprint)?;
        Ok(ConflictEvaluationRecord {
            id: row.try_get("id")?,
            status: parse_conflict_evaluation_status(
                row.try_get::<String, _>("evaluation_status")?.as_str(),
            )?,
            evaluation_fingerprint: existing,
            created_at: row.try_get("created_at")?,
        })
    }

    pub async fn append_conflict_event(
        &self,
        conflict_id: Uuid,
        event_type: &str,
        actor: &str,
        payload: &Value,
        idempotency_key: &str,
    ) -> PersistenceResult<()> {
        if !matches!(
            event_type,
            "resolved" | "reopened" | "dismissed" | "accepted_unknown"
        ) {
            return Err(PersistenceError::InvalidState(
                "冲突事件类型无效".to_string(),
            ));
        }
        let fingerprint = sha256_json(&json!({
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
        .bind(payload)
        .bind(idempotency_key)
        .bind(&fingerprint)
        .fetch_optional(&self.pool)
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
            .bind(idempotency_key)
            .fetch_one(&self.pool)
            .await?;
            ensure_fingerprint("冲突事件", idempotency_key, &existing, &fingerprint)?;
        }
        Ok(())
    }
}

fn parse_conflict_evaluation_status(value: &str) -> PersistenceResult<ConflictEvaluationStatus> {
    match value {
        "auto_resolved" => Ok(ConflictEvaluationStatus::AutoResolved),
        "manual_required" => Ok(ConflictEvaluationStatus::ManualRequired),
        "accepted_unknown" => Ok(ConflictEvaluationStatus::AcceptedUnknown),
        other => Err(PersistenceError::InvalidState(format!(
            "未知冲突评估状态：{other}"
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::{parse_conflict_evaluation_status, ConflictEvaluationStatus};
    use crate::PersistenceError;

    #[test]
    fn persisted_statuses_reject_unknown_case_and_padding() {
        for status in [
            ConflictEvaluationStatus::AutoResolved,
            ConflictEvaluationStatus::ManualRequired,
            ConflictEvaluationStatus::AcceptedUnknown,
        ] {
            assert_eq!(
                parse_conflict_evaluation_status(status.as_str()).unwrap(),
                status
            );
            for value in [
                status.as_str().to_uppercase(),
                format!(" {}", status.as_str()),
            ] {
                assert!(matches!(
                    parse_conflict_evaluation_status(&value),
                    Err(PersistenceError::InvalidState(_))
                ));
            }
        }
        assert!(matches!(
            parse_conflict_evaluation_status("future_status"),
            Err(PersistenceError::InvalidState(_))
        ));
    }
}
