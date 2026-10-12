use super::aggregation::aggregate_provider_rows;
use crate::{
    postmatch::{round6, PostmatchPartition},
    PersistenceResult, PostgresStore,
};
use football_domain::POSTMATCH_MONITORING_VERSION;
use uuid::Uuid;

impl PostgresStore {
    pub(crate) async fn refresh_provider_scores_for_partition(
        &self,
        partition: &PostmatchPartition,
    ) -> PersistenceResult<()> {
        let rows = sqlx::query(
            r#"
            SELECT item.provider_id, decision.id AS decision_id, decision.verdict,
                   decision.accuracy_score, decision.reliability_score,
                   item.timeliness_score, decision.decision_fingerprint
            FROM review.evidence_scoring_items item
            JOIN review.postmatch_settlements settlement ON settlement.id=item.settlement_id
            JOIN review.evidence_scoring_decisions decision ON decision.item_id=item.id
            WHERE settlement.competition_id=$1
              AND settlement.competition_profile_id=$2
              AND settlement.model_version_id=$3
              AND settlement.parameter_set_id=$4
              AND settlement.horizon=$5
              AND item.provider_id IS NOT NULL
            ORDER BY item.provider_id, decision.id
            "#,
        )
        .bind(partition.competition_id)
        .bind(partition.competition_profile_id)
        .bind(partition.model_version_id)
        .bind(partition.parameter_set_id)
        .bind(&partition.horizon)
        .fetch_all(&self.pool)
        .await?;
        let groups = aggregate_provider_rows(rows)?;
        for (provider_id, aggregate) in groups {
            let (accuracy_mean, timeliness_mean, reliability_mean, weighted_score) =
                aggregate.scores();
            let (decision_set_sha256, snapshot_key) =
                aggregate.snapshot_identity(provider_id, &partition.partition_key)?;
            sqlx::query(
                r#"
                INSERT INTO analytics.provider_score_snapshots (
                    id, provider_id, scope_key, competition_id, competition_profile_id,
                    model_version_id, parameter_set_id, horizon, sample_size,
                    correct_count, partial_count, incorrect_count, not_verifiable_count,
                    accuracy_mean, timeliness_mean, reliability_mean, weighted_score,
                    decision_set_sha256, snapshot_key, calculation_version
                ) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17,$18,$19,$20)
                ON CONFLICT (snapshot_key) DO NOTHING
                "#,
            )
            .bind(Uuid::new_v4())
            .bind(provider_id)
            .bind(&partition.partition_key)
            .bind(partition.competition_id)
            .bind(partition.competition_profile_id)
            .bind(partition.model_version_id)
            .bind(partition.parameter_set_id)
            .bind(&partition.horizon)
            .bind(aggregate.total() as i64)
            .bind(aggregate.correct as i64)
            .bind(aggregate.partial as i64)
            .bind(aggregate.incorrect as i64)
            .bind(aggregate.not_verifiable as i64)
            .bind(round6(accuracy_mean))
            .bind(round6(timeliness_mean))
            .bind(round6(reliability_mean))
            .bind(weighted_score)
            .bind(&decision_set_sha256)
            .bind(snapshot_key)
            .bind(POSTMATCH_MONITORING_VERSION)
            .execute(&self.pool)
            .await?;
        }
        Ok(())
    }
}
