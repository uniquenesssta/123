use crate::{postmatch::non_negative_u64, PersistenceResult, PostgresStore};
use football_domain::PostmatchOverview;

impl PostgresStore {
    pub async fn postmatch_overview(&self, limit: u32) -> PersistenceResult<PostmatchOverview> {
        let settlements = self.list_postmatch_settlements(limit).await?;
        let evidence_queue = self.list_evidence_scoring_items(None, limit).await?;
        let provider_scores = self.list_provider_scores(limit).await?;
        let drift_runs = self.list_postmatch_drift_runs(limit).await?;
        let settlement_count = non_negative_u64(
            sqlx::query_scalar::<_, i64>(
                "SELECT count(*)::bigint FROM review.postmatch_settlements",
            )
            .fetch_one(&self.pool)
            .await?,
            "赛后结算数量",
        )?;
        let pending_evidence_count = non_negative_u64(
            sqlx::query_scalar::<_, i64>(
                "SELECT count(*)::bigint FROM review.evidence_scoring_items item WHERE NOT EXISTS (SELECT 1 FROM review.evidence_scoring_decisions decision WHERE decision.item_id=item.id)",
            )
            .fetch_one(&self.pool)
            .await?,
            "待评分证据数量",
        )?;
        let scored_evidence_count = non_negative_u64(
            sqlx::query_scalar::<_, i64>(
                "SELECT count(*)::bigint FROM review.evidence_scoring_decisions WHERE verdict <> 'not_verifiable'",
            )
            .fetch_one(&self.pool)
            .await?,
            "已评分证据数量",
        )?;
        Ok(PostmatchOverview {
            settlement_count,
            pending_evidence_count,
            scored_evidence_count,
            settlements,
            evidence_queue,
            provider_scores,
            drift_runs,
        })
    }
}
