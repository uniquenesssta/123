use crate::PersistenceResult;
use football_domain::EvidenceScoringItemRecord;
use sqlx::Row;

pub(super) fn evidence_item_from_row(
    row: &sqlx::postgres::PgRow,
) -> PersistenceResult<EvidenceScoringItemRecord> {
    Ok(EvidenceScoringItemRecord {
        id: row.try_get("id")?,
        settlement_id: row.try_get("settlement_id")?,
        evidence_id: row.try_get("evidence_id")?,
        provider_id: row.try_get("provider_id")?,
        provider_name: row.try_get("provider_name")?,
        source_document_id: row.try_get("source_document_id")?,
        field_key: row.try_get("field_key")?,
        verification_state: row.try_get("verification_state")?,
        source_tier: row.try_get("source_tier")?,
        source_title: row.try_get("source_title")?,
        source_domain: row.try_get("source_domain")?,
        published_at: row.try_get("published_at")?,
        retrieved_at: row.try_get("retrieved_at")?,
        data_cutoff_at: row.try_get("data_cutoff_at")?,
        timeliness_score: row.try_get("timeliness_score")?,
        decision_id: row.try_get("decision_id")?,
        verdict: row.try_get("verdict")?,
        accuracy_score: row.try_get("accuracy_score")?,
        reliability_score: row.try_get("reliability_score")?,
        decided_by: row.try_get("decided_by")?,
        decision_note: row.try_get("decision_note")?,
        decided_at: row.try_get("decided_at")?,
        status: row.try_get("status")?,
        created_at: row.try_get("created_at")?,
    })
}
