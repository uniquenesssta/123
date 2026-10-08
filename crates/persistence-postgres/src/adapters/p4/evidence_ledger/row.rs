use crate::{PersistenceError, PersistenceResult};
use football_domain::{EvidenceClaimRecord, EvidenceVerificationState};
use sqlx::Row;

pub(super) fn evidence_claim_record_from_row(
    row: &sqlx::postgres::PgRow,
) -> PersistenceResult<EvidenceClaimRecord> {
    Ok(EvidenceClaimRecord {
        id: row.try_get("id")?,
        match_id: row.try_get("match_id")?,
        field_key: row.try_get("field_key")?,
        verification_state: parse_verification_state(
            row.try_get::<String, _>("verification_state")?.as_str(),
        )?,
        content_sha256: row.try_get("content_sha256")?,
        claim_fingerprint: row.try_get("claim_fingerprint")?,
        idempotency_key: row.try_get("idempotency_key")?,
        created_at: row.try_get("created_at")?,
    })
}

pub(crate) fn parse_verification_state(
    value: &str,
) -> PersistenceResult<EvidenceVerificationState> {
    match value {
        "CONFIRMED" => Ok(EvidenceVerificationState::Confirmed),
        "PROBABLE" => Ok(EvidenceVerificationState::Probable),
        "CONFLICT" => Ok(EvidenceVerificationState::Conflict),
        "NOT_FOUND" => Ok(EvidenceVerificationState::NotFound),
        "STALE" => Ok(EvidenceVerificationState::Stale),
        "NOT_APPLICABLE" => Ok(EvidenceVerificationState::NotApplicable),
        other => Err(PersistenceError::InvalidState(format!(
            "未知证据验证状态：{other}"
        ))),
    }
}
