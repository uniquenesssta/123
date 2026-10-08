use super::fingerprint::ensure_fingerprint;
use crate::{sha256_json, PersistenceError, PersistenceResult, PostgresStore};
use football_domain::{EntityResolutionDraft, EntityResolutionRecord, EntityResolutionStatus};
use serde_json::json;
use sqlx::Row;
use uuid::Uuid;

impl PostgresStore {
    pub async fn append_entity_resolution(
        &self,
        draft: &EntityResolutionDraft,
    ) -> PersistenceResult<EntityResolutionRecord> {
        let fingerprint = sha256_json(&json!({
            "research_run_id": draft.research_run_id,
            "match_id": draft.match_id,
            "trace_id": draft.trace_id,
            "fact_key": draft.fact_key,
            "entity_type": draft.entity_type,
            "raw_name": draft.raw_name,
            "normalized_name": draft.normalized_name,
            "external_id": draft.external_id,
            "status": draft.status.as_str(),
            "resolved_entity_id": draft.resolved_entity_id,
            "resolved_name": draft.resolved_name,
            "strategy": draft.strategy,
            "confidence_score": draft.confidence_score,
            "candidates": draft.candidates,
            "reason": draft.reason,
        }))?;
        let row = sqlx::query(
            r#"
            INSERT INTO research.entity_resolutions (
                id, research_run_id, match_id, trace_id, fact_key,
                entity_type, raw_name, normalized_name, external_id,
                resolution_status, resolved_entity_id, resolved_name,
                strategy, confidence_score, candidates, reason,
                idempotency_key, resolution_fingerprint
            ) VALUES (
                $1, $2, $3, $4, $5,
                $6, $7, $8, $9,
                $10, $11, $12,
                $13, $14, $15, $16,
                $17, $18
            )
            ON CONFLICT (idempotency_key) DO NOTHING
            RETURNING id, resolution_status, resolved_entity_id,
                      resolution_fingerprint, created_at
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(draft.research_run_id)
        .bind(draft.match_id)
        .bind(draft.trace_id)
        .bind(&draft.fact_key)
        .bind(&draft.entity_type)
        .bind(&draft.raw_name)
        .bind(&draft.normalized_name)
        .bind(&draft.external_id)
        .bind(draft.status.as_str())
        .bind(draft.resolved_entity_id)
        .bind(&draft.resolved_name)
        .bind(&draft.strategy)
        .bind(i32::from(draft.confidence_score))
        .bind(serde_json::to_value(&draft.candidates)?)
        .bind(&draft.reason)
        .bind(&draft.idempotency_key)
        .bind(&fingerprint)
        .fetch_optional(&self.pool)
        .await?;
        let row = match row {
            Some(row) => row,
            None => {
                sqlx::query(
                    r#"SELECT id, resolution_status, resolved_entity_id,
                          resolution_fingerprint, created_at
                   FROM research.entity_resolutions
                   WHERE idempotency_key = $1"#,
                )
                .bind(&draft.idempotency_key)
                .fetch_one(&self.pool)
                .await?
            }
        };
        let existing: String = row.try_get("resolution_fingerprint")?;
        ensure_fingerprint("实体解析", &draft.idempotency_key, &existing, &fingerprint)?;
        Ok(EntityResolutionRecord {
            id: row.try_get("id")?,
            status: parse_entity_resolution_status(
                row.try_get::<String, _>("resolution_status")?.as_str(),
            )?,
            resolved_entity_id: row.try_get("resolved_entity_id")?,
            resolution_fingerprint: existing,
            created_at: row.try_get("created_at")?,
        })
    }
}

fn parse_entity_resolution_status(value: &str) -> PersistenceResult<EntityResolutionStatus> {
    match value {
        "resolved" => Ok(EntityResolutionStatus::Resolved),
        "ambiguous" => Ok(EntityResolutionStatus::Ambiguous),
        "unmatched" => Ok(EntityResolutionStatus::Unmatched),
        "unsupported" => Ok(EntityResolutionStatus::Unsupported),
        other => Err(PersistenceError::InvalidState(format!(
            "未知实体解析状态：{other}"
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::{parse_entity_resolution_status, EntityResolutionStatus};
    use crate::PersistenceError;

    #[test]
    fn persisted_statuses_reject_unknown_case_and_padding() {
        for status in [
            EntityResolutionStatus::Resolved,
            EntityResolutionStatus::Ambiguous,
            EntityResolutionStatus::Unmatched,
            EntityResolutionStatus::Unsupported,
        ] {
            assert_eq!(
                parse_entity_resolution_status(status.as_str()).unwrap(),
                status
            );
            for value in [
                status.as_str().to_uppercase(),
                format!(" {}", status.as_str()),
            ] {
                assert!(matches!(
                    parse_entity_resolution_status(&value),
                    Err(PersistenceError::InvalidState(_))
                ));
            }
        }
        assert!(matches!(
            parse_entity_resolution_status("future_status"),
            Err(PersistenceError::InvalidState(_))
        ));
    }
}
