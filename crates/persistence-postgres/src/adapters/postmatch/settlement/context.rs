use crate::{PersistenceError, PersistenceResult, PostgresStore};
use chrono::{DateTime, Utc};
use serde_json::Value;
use sqlx::Row;
use uuid::Uuid;

#[derive(Debug)]
pub(super) struct SettlementContext {
    pub(super) match_review_id: Uuid,
    pub(super) match_id: Uuid,
    pub(super) match_key: String,
    pub(super) home_team_name: String,
    pub(super) away_team_name: String,
    pub(super) competition_id: Option<Uuid>,
    pub(super) review_status: String,
    pub(super) data_coverage: f64,
    pub(super) prediction_evaluation: Value,
    pub(super) model_run_id: Option<Uuid>,
    pub(super) run_status: Option<String>,
    pub(super) feature_snapshot_id: Option<Uuid>,
    pub(super) competition_profile_id: Option<Uuid>,
    pub(super) model_version_id: Option<Uuid>,
    pub(super) parameter_set_id: Option<Uuid>,
    pub(super) rule_package_id: Option<Uuid>,
    pub(super) horizon: Option<String>,
    pub(super) kickoff_time: DateTime<Utc>,
    pub(super) home_goals_90: Option<i16>,
    pub(super) away_goals_90: Option<i16>,
    pub(super) result_finalized_at: Option<DateTime<Utc>>,
    pub(super) snapshot_cutoff_at: Option<DateTime<Utc>>,
    pub(super) snapshot_match_id: Option<Uuid>,
    pub(super) snapshot_model_version_id: Option<Uuid>,
    pub(super) snapshot_parameter_set_id: Option<Uuid>,
    pub(super) snapshot_competition_profile_id: Option<Uuid>,
    pub(super) snapshot_source_kind: Option<String>,
    pub(super) snapshot_evidence_scope: Option<String>,
    pub(super) existing_settlement_id: Option<Uuid>,
}

impl PostgresStore {
    pub(super) async fn load_settlement_context(
        &self,
        match_review_id: Uuid,
    ) -> PersistenceResult<SettlementContext> {
        let row = sqlx::query(
            r#"
            SELECT review.id AS match_review_id, review.match_id, fixture.external_key AS match_key,
                   home.canonical_name AS home_team_name, away.canonical_name AS away_team_name,
                   fixture.competition_id,
                   review.status AS review_status, review.data_coverage::double precision AS data_coverage,
                   review.prediction_evaluation, review.source_run_id AS model_run_id,
                   run.status AS run_status, run.feature_snapshot_id,
                   package.competition_profile_id, run.model_version_id,
                   run.parameter_set_id, run.rule_package_id,
                   run.snapshot_type AS horizon, fixture.kickoff_time,
                   result.home_goals_90, result.away_goals_90, result.finalized_at AS result_finalized_at,
                   snapshot.data_cutoff_time AS snapshot_cutoff_at,
                   snapshot.match_id AS snapshot_match_id,
                   snapshot.model_version_id AS snapshot_model_version_id,
                   snapshot.parameter_set_id AS snapshot_parameter_set_id,
                   snapshot.competition_profile_id AS snapshot_competition_profile_id,
                   snapshot.source_kind AS snapshot_source_kind,
                   snapshot.evidence_scope AS snapshot_evidence_scope,
                   settlement.id AS existing_settlement_id
            FROM review.match_reviews review
            JOIN football.matches fixture ON fixture.id = review.match_id
            JOIN football.teams home ON home.id = fixture.home_team_id
            JOIN football.teams away ON away.id = fixture.away_team_id
            LEFT JOIN football.match_results result ON result.match_id = fixture.id
            LEFT JOIN model.runs run ON run.id = review.source_run_id
            LEFT JOIN feature.snapshots snapshot ON snapshot.id = run.feature_snapshot_id
            LEFT JOIN model.rule_packages package ON package.id = run.rule_package_id
            LEFT JOIN review.postmatch_settlements settlement
              ON settlement.match_review_id = review.id AND settlement.model_run_id = run.id
            WHERE review.id=$1
            "#,
        )
        .bind(match_review_id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| PersistenceError::InvalidState("复盘记录不存在".to_string()))?;
        Ok(SettlementContext {
            match_review_id: row.try_get("match_review_id")?,
            match_id: row.try_get("match_id")?,
            match_key: row.try_get("match_key")?,
            home_team_name: row.try_get("home_team_name")?,
            away_team_name: row.try_get("away_team_name")?,
            competition_id: row.try_get("competition_id")?,
            review_status: row.try_get("review_status")?,
            data_coverage: row.try_get("data_coverage")?,
            prediction_evaluation: row.try_get("prediction_evaluation")?,
            model_run_id: row.try_get("model_run_id")?,
            run_status: row.try_get("run_status")?,
            feature_snapshot_id: row.try_get("feature_snapshot_id")?,
            competition_profile_id: row.try_get("competition_profile_id")?,
            model_version_id: row.try_get("model_version_id")?,
            parameter_set_id: row.try_get("parameter_set_id")?,
            rule_package_id: row.try_get("rule_package_id")?,
            horizon: row.try_get("horizon")?,
            kickoff_time: row.try_get("kickoff_time")?,
            home_goals_90: row.try_get("home_goals_90")?,
            away_goals_90: row.try_get("away_goals_90")?,
            result_finalized_at: row.try_get("result_finalized_at")?,
            snapshot_cutoff_at: row.try_get("snapshot_cutoff_at")?,
            snapshot_match_id: row.try_get("snapshot_match_id")?,
            snapshot_model_version_id: row.try_get("snapshot_model_version_id")?,
            snapshot_parameter_set_id: row.try_get("snapshot_parameter_set_id")?,
            snapshot_competition_profile_id: row.try_get("snapshot_competition_profile_id")?,
            snapshot_source_kind: row.try_get("snapshot_source_kind")?,
            snapshot_evidence_scope: row.try_get("snapshot_evidence_scope")?,
            existing_settlement_id: row.try_get("existing_settlement_id")?,
        })
    }
}
