use super::policy::{validate_evidence_decision_note, verdict_scores};
use crate::{sha256_json, write_audit_event, PersistenceError, PersistenceResult, PostgresStore};
use football_domain::{EvidenceScoringDecisionDraft, EvidenceScoringItemRecord};
use serde_json::json;
use sqlx::Row;
use uuid::Uuid;

impl PostgresStore {
    pub async fn decide_evidence_scoring_item(
        &self,
        draft: &EvidenceScoringDecisionDraft,
    ) -> PersistenceResult<EvidenceScoringItemRecord> {
        validate_evidence_decision_note(&draft.decision_note)?;
        let item = sqlx::query(
            r#"
            SELECT item.id, item.verification_state, item.source_tier,
                   EXISTS (SELECT 1 FROM review.evidence_scoring_decisions decision WHERE decision.item_id=item.id) AS decided
            FROM review.evidence_scoring_items item
            WHERE item.id=$1
            "#,
        )
        .bind(draft.item_id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| PersistenceError::InvalidState("证据评分项不存在".to_string()))?;
        if item.try_get::<bool, _>("decided")? {
            return Err(PersistenceError::InvalidState(
                "该证据评分项已经完成判定，历史决定不可覆盖".to_string(),
            ));
        }
        let verification_state: String = item.try_get("verification_state")?;
        let source_tier: String = item.try_get("source_tier")?;
        let (accuracy_score, reliability_score) =
            verdict_scores(draft.verdict, &verification_state, &source_tier);
        let decision_fingerprint = sha256_json(&json!({
            "item_id": draft.item_id,
            "verdict": draft.verdict.as_str(),
            "accuracy_score": accuracy_score,
            "reliability_score": reliability_score,
            "decided_by": draft.decided_by.as_deref(),
            "decision_note": draft.decision_note.trim(),
        }))?;
        let decision_id = Uuid::new_v4();
        let mut tx = self.pool.begin().await?;
        sqlx::query(
            r#"
            INSERT INTO review.evidence_scoring_decisions (
                id, item_id, verdict, accuracy_score, reliability_score,
                decided_by, decision_note, decision_fingerprint
            ) VALUES ($1,$2,$3,$4,$5,$6,$7,$8)
            "#,
        )
        .bind(decision_id)
        .bind(draft.item_id)
        .bind(draft.verdict.as_str())
        .bind(accuracy_score)
        .bind(reliability_score)
        .bind(draft.decided_by.as_deref())
        .bind(draft.decision_note.trim())
        .bind(&decision_fingerprint)
        .execute(&mut *tx)
        .await?;
        write_audit_event(
            &mut tx,
            "evidence_scoring_decided",
            "evidence_scoring_item",
            Some(draft.item_id.to_string()),
            json!({
                "decision_id": decision_id,
                "verdict": draft.verdict.as_str(),
                "accuracy_score": accuracy_score,
                "reliability_score": reliability_score,
            }),
        )
        .await?;
        tx.commit().await?;
        self.read_evidence_scoring_item(draft.item_id).await
    }
}
