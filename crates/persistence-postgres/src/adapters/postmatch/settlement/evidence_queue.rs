use crate::{postmatch::round6, sha256_json, PersistenceError, PersistenceResult, PostgresStore};
use chrono::{DateTime, Utc};
use serde_json::json;
use sqlx::{Postgres, Row, Transaction};
use uuid::Uuid;

impl PostgresStore {
    pub(super) async fn insert_evidence_scoring_items_in_tx(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        settlement_id: Uuid,
        feature_snapshot_id: Uuid,
        data_cutoff_at: DateTime<Utc>,
    ) -> PersistenceResult<()> {
        let rows = sqlx::query(
            r#"
            SELECT DISTINCT claim.id AS evidence_id, document.provider_id,
                   claim.source_document_id, link.field_key, claim.verification_state,
                   claim.source_tier, claim.published_at, claim.retrieved_at
            FROM feature.snapshot_evidence link
            JOIN research.evidence_claims claim ON claim.id = link.evidence_id
            LEFT JOIN catalog.source_documents document ON document.id = claim.source_document_id
            WHERE link.snapshot_id=$1
            ORDER BY claim.id
            "#,
        )
        .bind(feature_snapshot_id)
        .fetch_all(&mut **tx)
        .await?;
        for row in rows {
            let evidence_id: Uuid = row.try_get("evidence_id")?;
            let provider_id: Option<Uuid> = row.try_get("provider_id")?;
            let source_document_id: Option<Uuid> = row.try_get("source_document_id")?;
            let field_key: String = row.try_get("field_key")?;
            let verification_state: String = row.try_get("verification_state")?;
            let source_tier: String = row.try_get("source_tier")?;
            let published_at: Option<DateTime<Utc>> = row.try_get("published_at")?;
            let retrieved_at: DateTime<Utc> = row.try_get("retrieved_at")?;
            if retrieved_at > data_cutoff_at
                || published_at
                    .as_ref()
                    .is_some_and(|published| published > &data_cutoff_at)
            {
                return Err(PersistenceError::InvalidState(format!(
                    "证据 {evidence_id} 超出冻结快照截止时间，不能进入正式评分队列"
                )));
            }
            let timeliness_score =
                calculate_timeliness_score(published_at.as_ref(), &retrieved_at, &data_cutoff_at);
            let item_fingerprint = sha256_json(&json!({
                "settlement_id": settlement_id,
                "evidence_id": evidence_id,
                "provider_id": provider_id,
                "field_key": &field_key,
                "verification_state": &verification_state,
                "source_tier": &source_tier,
                "published_at": published_at.as_ref(),
                "retrieved_at": &retrieved_at,
                "data_cutoff_at": &data_cutoff_at,
                "timeliness_score": timeliness_score,
            }))?;
            sqlx::query(
                r#"
                INSERT INTO review.evidence_scoring_items (
                    id, settlement_id, evidence_id, provider_id, source_document_id,
                    field_key, verification_state, source_tier, published_at,
                    retrieved_at, data_cutoff_at, timeliness_score, item_fingerprint
                ) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13)
                ON CONFLICT (settlement_id, evidence_id, field_key) DO NOTHING
                "#,
            )
            .bind(Uuid::new_v4())
            .bind(settlement_id)
            .bind(evidence_id)
            .bind(provider_id)
            .bind(source_document_id)
            .bind(&field_key)
            .bind(&verification_state)
            .bind(&source_tier)
            .bind(published_at.as_ref())
            .bind(retrieved_at)
            .bind(data_cutoff_at)
            .bind(timeliness_score)
            .bind(&item_fingerprint)
            .execute(&mut **tx)
            .await?;
        }
        Ok(())
    }
}

fn calculate_timeliness_score(
    published_at: Option<&DateTime<Utc>>,
    retrieved_at: &DateTime<Utc>,
    cutoff_at: &DateTime<Utc>,
) -> f64 {
    let reference = published_at.unwrap_or(retrieved_at);
    if reference > cutoff_at || retrieved_at > cutoff_at {
        return 0.0;
    }
    let age_hours = cutoff_at
        .signed_duration_since(*reference)
        .num_minutes()
        .max(0) as f64
        / 60.0;
    let score = if age_hours <= 6.0 {
        1.0
    } else if age_hours <= 24.0 {
        0.9
    } else if age_hours <= 72.0 {
        0.75
    } else if age_hours <= 168.0 {
        0.5
    } else {
        0.25
    };
    round6(score)
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn timeliness_is_bounded_and_rewards_fresh_sources() {
        let cutoff = Utc.with_ymd_and_hms(2026, 1, 2, 12, 0, 0).unwrap();
        let fresh = Utc.with_ymd_and_hms(2026, 1, 2, 9, 0, 0).unwrap();
        let old = Utc.with_ymd_and_hms(2025, 12, 20, 9, 0, 0).unwrap();
        assert_eq!(
            calculate_timeliness_score(Some(&fresh), &fresh, &cutoff),
            1.0
        );
        assert_eq!(calculate_timeliness_score(Some(&old), &old, &cutoff), 0.25);
        assert_eq!(
            calculate_timeliness_score(Some(&cutoff), &cutoff, &cutoff),
            1.0
        );
    }
}
