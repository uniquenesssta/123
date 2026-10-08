use super::fingerprint::ensure_fingerprint;
use crate::{sha256_json, PersistenceError, PersistenceResult, PostgresStore};
use football_domain::{TimeAuditDraft, TimeAuditRecord, TimeAuditStatus};
use serde_json::json;
use sqlx::Row;
use uuid::Uuid;

impl PostgresStore {
    pub async fn append_time_audit(
        &self,
        draft: &TimeAuditDraft,
    ) -> PersistenceResult<TimeAuditRecord> {
        let fingerprint = sha256_json(&json!({
            "research_run_id": draft.research_run_id,
            "match_id": draft.match_id,
            "trace_id": draft.trace_id,
            "fact_key": draft.fact_key,
            "field_key": draft.field_key,
            "data_cutoff_at": draft.data_cutoff_at,
            "published_at": draft.published_at,
            "observed_at": draft.observed_at,
            "effective_at": draft.effective_at,
            "retrieved_at": draft.retrieved_at,
            "timezone": draft.timezone,
            "status": draft.status.as_str(),
            "reason": draft.reason,
        }))?;
        let row = sqlx::query(
            r#"
            INSERT INTO research.time_audits (
                id, research_run_id, match_id, trace_id, fact_key, field_key,
                data_cutoff_at, published_at, observed_at, effective_at,
                retrieved_at, timezone, audit_status, accepted, reason,
                idempotency_key, time_fingerprint
            ) VALUES (
                $1, $2, $3, $4, $5, $6,
                $7, $8, $9, $10,
                $11, $12, $13, $14, $15,
                $16, $17
            )
            ON CONFLICT (idempotency_key) DO NOTHING
            RETURNING id, audit_status, time_fingerprint, created_at
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(draft.research_run_id)
        .bind(draft.match_id)
        .bind(draft.trace_id)
        .bind(&draft.fact_key)
        .bind(&draft.field_key)
        .bind(draft.data_cutoff_at)
        .bind(draft.published_at)
        .bind(draft.observed_at)
        .bind(draft.effective_at)
        .bind(draft.retrieved_at)
        .bind(&draft.timezone)
        .bind(draft.status.as_str())
        .bind(draft.status.accepted())
        .bind(&draft.reason)
        .bind(&draft.idempotency_key)
        .bind(&fingerprint)
        .fetch_optional(&self.pool)
        .await?;
        let row = match row {
            Some(row) => row,
            None => {
                sqlx::query(
                    r#"SELECT id, audit_status, time_fingerprint, created_at
                   FROM research.time_audits
                   WHERE idempotency_key = $1"#,
                )
                .bind(&draft.idempotency_key)
                .fetch_one(&self.pool)
                .await?
            }
        };
        let existing: String = row.try_get("time_fingerprint")?;
        ensure_fingerprint("时间审计", &draft.idempotency_key, &existing, &fingerprint)?;
        Ok(TimeAuditRecord {
            id: row.try_get("id")?,
            status: parse_time_audit_status(row.try_get::<String, _>("audit_status")?.as_str())?,
            time_fingerprint: existing,
            created_at: row.try_get("created_at")?,
        })
    }
}

fn parse_time_audit_status(value: &str) -> PersistenceResult<TimeAuditStatus> {
    match value {
        "accepted" => Ok(TimeAuditStatus::Accepted),
        "accepted_non_fact" => Ok(TimeAuditStatus::AcceptedNonFact),
        "rejected_future" => Ok(TimeAuditStatus::RejectedFuture),
        "rejected_retrieved_after_cutoff" => Ok(TimeAuditStatus::RejectedRetrievedAfterCutoff),
        "rejected_missing_evidence_time" => Ok(TimeAuditStatus::RejectedMissingEvidenceTime),
        "rejected_missing_timezone" => Ok(TimeAuditStatus::RejectedMissingTimezone),
        "rejected_invalid_order" => Ok(TimeAuditStatus::RejectedInvalidOrder),
        other => Err(PersistenceError::InvalidState(format!(
            "未知时间审计状态：{other}"
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::{parse_time_audit_status, TimeAuditStatus};
    use crate::PersistenceError;

    #[test]
    fn persisted_statuses_reject_unknown_case_and_padding() {
        for status in [
            TimeAuditStatus::Accepted,
            TimeAuditStatus::AcceptedNonFact,
            TimeAuditStatus::RejectedFuture,
            TimeAuditStatus::RejectedRetrievedAfterCutoff,
            TimeAuditStatus::RejectedMissingEvidenceTime,
            TimeAuditStatus::RejectedMissingTimezone,
            TimeAuditStatus::RejectedInvalidOrder,
        ] {
            assert_eq!(parse_time_audit_status(status.as_str()).unwrap(), status);
            for value in [
                status.as_str().to_uppercase(),
                format!(" {}", status.as_str()),
            ] {
                assert!(matches!(
                    parse_time_audit_status(&value),
                    Err(PersistenceError::InvalidState(_))
                ));
            }
        }
        assert!(matches!(
            parse_time_audit_status("future_status"),
            Err(PersistenceError::InvalidState(_))
        ));
    }
}
