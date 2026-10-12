use super::{mapping::evidence_item_from_row, policy::validate_evidence_status};
use crate::{PersistenceError, PersistenceResult, PostgresStore};
use football_domain::EvidenceScoringItemRecord;
use uuid::Uuid;

impl PostgresStore {
    pub async fn list_evidence_scoring_items(
        &self,
        status: Option<&str>,
        limit: u32,
    ) -> PersistenceResult<Vec<EvidenceScoringItemRecord>> {
        validate_evidence_status(status)?;
        let rows = sqlx::query(
            r#"
            SELECT item.id, item.settlement_id, item.evidence_id, item.provider_id,
                   provider.name AS provider_name, item.source_document_id, item.field_key,
                   item.verification_state, item.source_tier, claim.source_title,
                   claim.source_domain, item.published_at, item.retrieved_at,
                   item.data_cutoff_at, item.timeliness_score, item.created_at,
                   decision.id AS decision_id, decision.verdict, decision.accuracy_score,
                   decision.reliability_score, decision.decided_by, decision.decision_note,
                   decision.decided_at,
                   CASE
                       WHEN decision.id IS NULL THEN 'pending'
                       WHEN decision.verdict = 'not_verifiable' THEN 'not_verifiable'
                       ELSE 'scored'
                   END AS status
            FROM review.evidence_scoring_items item
            JOIN research.evidence_claims claim ON claim.id = item.evidence_id
            LEFT JOIN catalog.data_providers provider ON provider.id = item.provider_id
            LEFT JOIN review.evidence_scoring_decisions decision ON decision.item_id = item.id
            WHERE $1::text IS NULL OR
                  ($1 = 'pending' AND decision.id IS NULL) OR
                  ($1 = 'scored' AND decision.id IS NOT NULL AND decision.verdict <> 'not_verifiable') OR
                  ($1 = 'not_verifiable' AND decision.verdict = 'not_verifiable')
            ORDER BY CASE WHEN decision.id IS NULL THEN 0 ELSE 1 END,
                     item.created_at DESC, item.id DESC
            LIMIT $2
            "#,
        )
        .bind(status)
        .bind(i64::from(limit.clamp(1, 1000)))
        .fetch_all(&self.pool)
        .await?;
        rows.iter().map(evidence_item_from_row).collect()
    }

    pub(super) async fn read_evidence_scoring_item(
        &self,
        item_id: Uuid,
    ) -> PersistenceResult<EvidenceScoringItemRecord> {
        let row = sqlx::query(
            r#"
            SELECT item.id, item.settlement_id, item.evidence_id, item.provider_id,
                   provider.name AS provider_name, item.source_document_id, item.field_key,
                   item.verification_state, item.source_tier, claim.source_title,
                   claim.source_domain, item.published_at, item.retrieved_at,
                   item.data_cutoff_at, item.timeliness_score, item.created_at,
                   decision.id AS decision_id, decision.verdict, decision.accuracy_score,
                   decision.reliability_score, decision.decided_by, decision.decision_note,
                   decision.decided_at,
                   CASE
                       WHEN decision.id IS NULL THEN 'pending'
                       WHEN decision.verdict = 'not_verifiable' THEN 'not_verifiable'
                       ELSE 'scored'
                   END AS status
            FROM review.evidence_scoring_items item
            JOIN research.evidence_claims claim ON claim.id = item.evidence_id
            LEFT JOIN catalog.data_providers provider ON provider.id = item.provider_id
            LEFT JOIN review.evidence_scoring_decisions decision ON decision.item_id = item.id
            WHERE item.id=$1
            "#,
        )
        .bind(item_id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| PersistenceError::InvalidState("证据评分项不存在".to_string()))?;
        evidence_item_from_row(&row)
    }
}
