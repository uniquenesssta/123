use super::mapping::provider_score_from_row;
use crate::{PersistenceResult, PostgresStore};
use football_domain::ProviderScoreSnapshotRecord;

impl PostgresStore {
    pub(crate) async fn list_provider_scores(
        &self,
        limit: u32,
    ) -> PersistenceResult<Vec<ProviderScoreSnapshotRecord>> {
        let rows = sqlx::query(
            r#"
            SELECT DISTINCT ON (score.provider_id, score.scope_key)
                   score.id, score.provider_id, provider.name AS provider_name,
                   score.scope_key, score.competition_id, score.competition_profile_id,
                   score.model_version_id, score.parameter_set_id, score.horizon,
                   score.sample_size, score.correct_count,
                   score.partial_count, score.incorrect_count, score.not_verifiable_count,
                   score.accuracy_mean, score.timeliness_mean, score.reliability_mean,
                   score.weighted_score, score.decision_set_sha256,
                   score.calculation_version, score.generated_at
            FROM analytics.provider_score_snapshots score
            JOIN catalog.data_providers provider ON provider.id=score.provider_id
            ORDER BY score.provider_id, score.scope_key, score.generated_at DESC, score.id DESC
            LIMIT $1
            "#,
        )
        .bind(i64::from(limit.clamp(1, 500)))
        .fetch_all(&self.pool)
        .await?;
        rows.iter().map(provider_score_from_row).collect()
    }
}
