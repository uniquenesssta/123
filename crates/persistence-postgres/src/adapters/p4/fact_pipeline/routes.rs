use super::fingerprint::ensure_fingerprint;
use crate::{sha256_json, PersistenceError, PersistenceResult, PostgresStore};
use football_domain::{EvidenceRouteDraft, EvidenceRouteRecord, EvidenceRouteStatus};
use serde_json::json;
use sqlx::Row;
use uuid::Uuid;

impl PostgresStore {
    pub async fn append_evidence_route(
        &self,
        draft: &EvidenceRouteDraft,
    ) -> PersistenceResult<EvidenceRouteRecord> {
        let mut evidence_ids = draft.selected_evidence_ids.clone();
        evidence_ids.sort_unstable();
        evidence_ids.dedup();
        let fingerprint = sha256_json(&json!({
            "research_run_id": draft.research_run_id,
            "match_id": draft.match_id,
            "trace_id": draft.trace_id,
            "route_key": draft.route_key,
            "field_key": draft.field_key,
            "target_module": draft.target_module,
            "target_slot": draft.target_slot,
            "route_registry_version": draft.route_registry_version,
            "entity_type": draft.entity_type,
            "entity_id": draft.entity_id,
            "status": draft.status.as_str(),
            "verification_state": draft.verification_state,
            "selected_evidence_ids": evidence_ids,
            "selected_value": draft.selected_value,
            "reason": draft.reason,
        }))?;
        let row = sqlx::query(
            r#"
            INSERT INTO research.evidence_routes (
                id, research_run_id, match_id, trace_id, route_key,
                field_key, target_module, target_slot, route_registry_version,
                entity_type, entity_id, route_status, verification_state, selected_evidence_ids,
                selected_value, reason, idempotency_key, route_fingerprint
            ) VALUES (
                $1, $2, $3, $4, $5,
                $6, $7, $8, $9,
                $10, $11, $12, $13, $14,
                $15, $16, $17, $18
            )
            ON CONFLICT (idempotency_key) DO NOTHING
            RETURNING id, route_status, route_fingerprint, created_at
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(draft.research_run_id)
        .bind(draft.match_id)
        .bind(draft.trace_id)
        .bind(&draft.route_key)
        .bind(&draft.field_key)
        .bind(&draft.target_module)
        .bind(&draft.target_slot)
        .bind(&draft.route_registry_version)
        .bind(&draft.entity_type)
        .bind(draft.entity_id)
        .bind(draft.status.as_str())
        .bind(&draft.verification_state)
        .bind(&evidence_ids)
        .bind(&draft.selected_value)
        .bind(&draft.reason)
        .bind(&draft.idempotency_key)
        .bind(&fingerprint)
        .fetch_optional(&self.pool)
        .await?;
        let row = match row {
            Some(row) => row,
            None => {
                sqlx::query(
                    r#"SELECT id, route_status, route_fingerprint, created_at
                   FROM research.evidence_routes
                   WHERE idempotency_key = $1"#,
                )
                .bind(&draft.idempotency_key)
                .fetch_one(&self.pool)
                .await?
            }
        };
        let existing: String = row.try_get("route_fingerprint")?;
        ensure_fingerprint("证据路由", &draft.idempotency_key, &existing, &fingerprint)?;
        Ok(EvidenceRouteRecord {
            id: row.try_get("id")?,
            status: parse_evidence_route_status(
                row.try_get::<String, _>("route_status")?.as_str(),
            )?,
            route_fingerprint: existing,
            created_at: row.try_get("created_at")?,
        })
    }
}

fn parse_evidence_route_status(value: &str) -> PersistenceResult<EvidenceRouteStatus> {
    match value {
        "routed" => Ok(EvidenceRouteStatus::Routed),
        "missing" => Ok(EvidenceRouteStatus::Missing),
        "blocked_entity" => Ok(EvidenceRouteStatus::BlockedEntity),
        "blocked_time" => Ok(EvidenceRouteStatus::BlockedTime),
        "blocked_conflict" => Ok(EvidenceRouteStatus::BlockedConflict),
        "blocked_unregistered_field" => Ok(EvidenceRouteStatus::BlockedUnregisteredField),
        "ignored_non_model_fact" => Ok(EvidenceRouteStatus::IgnoredNonModelFact),
        other => Err(PersistenceError::InvalidState(format!(
            "未知证据路由状态：{other}"
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::{parse_evidence_route_status, EvidenceRouteStatus};
    use crate::PersistenceError;

    #[test]
    fn persisted_statuses_reject_unknown_case_and_padding() {
        for status in [
            EvidenceRouteStatus::Routed,
            EvidenceRouteStatus::Missing,
            EvidenceRouteStatus::BlockedEntity,
            EvidenceRouteStatus::BlockedTime,
            EvidenceRouteStatus::BlockedConflict,
            EvidenceRouteStatus::BlockedUnregisteredField,
            EvidenceRouteStatus::IgnoredNonModelFact,
        ] {
            assert_eq!(
                parse_evidence_route_status(status.as_str()).unwrap(),
                status
            );
            for value in [
                status.as_str().to_uppercase(),
                format!(" {}", status.as_str()),
            ] {
                assert!(matches!(
                    parse_evidence_route_status(&value),
                    Err(PersistenceError::InvalidState(_))
                ));
            }
        }
        assert!(matches!(
            parse_evidence_route_status("future_status"),
            Err(PersistenceError::InvalidState(_))
        ));
    }
}
