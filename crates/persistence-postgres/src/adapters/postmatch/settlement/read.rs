use crate::{postmatch::non_negative_u64, PersistenceError, PersistenceResult, PostgresStore};
use football_domain::PostmatchSettlementRecord;
use sqlx::Row;
use uuid::Uuid;

impl PostgresStore {
    pub async fn list_postmatch_settlements(
        &self,
        limit: u32,
    ) -> PersistenceResult<Vec<PostmatchSettlementRecord>> {
        let rows = sqlx::query(&settlement_select_sql(
            "ORDER BY settlement.settled_at DESC, settlement.id DESC LIMIT $1",
        ))
        .bind(i64::from(limit.clamp(1, 500)))
        .fetch_all(&self.pool)
        .await?;
        rows.iter().map(settlement_from_row).collect()
    }

    pub(super) async fn read_postmatch_settlement(
        &self,
        settlement_id: Uuid,
    ) -> PersistenceResult<PostmatchSettlementRecord> {
        let row = sqlx::query(&settlement_select_sql("WHERE settlement.id=$1"))
            .bind(settlement_id)
            .fetch_optional(&self.pool)
            .await?
            .ok_or_else(|| PersistenceError::InvalidState("赛后结算记录不存在".to_string()))?;
        settlement_from_row(&row)
    }
}

fn settlement_select_sql(suffix: &str) -> String {
    format!(
        r#"
        SELECT settlement.id, settlement.match_id, settlement.match_review_id,
               settlement.model_run_id, settlement.feature_snapshot_id,
               settlement.competition_id, competition.name AS competition_name,
               settlement.competition_profile_id, settlement.model_version_id,
               version.version AS model_version, settlement.parameter_set_id,
               parameter.parameter_version, settlement.rule_package_id,
               settlement.horizon, fixture.external_key AS match_key,
               home.canonical_name AS home_team_name,
               away.canonical_name AS away_team_name,
               settlement.home_goals_90, settlement.away_goals_90,
               settlement.result_finalized_at, settlement.result_fingerprint, settlement.settlement_key,
               settlement.settlement_version, settlement.status,
               settlement.settled_by, settlement.settlement_note,
               settlement.metadata, settlement.settled_at,
               COALESCE(evidence_counts.item_count,0)::bigint AS evidence_item_count,
               COALESCE(evidence_counts.scored_count,0)::bigint AS scored_evidence_count,
               drift.status AS drift_status
        FROM review.postmatch_settlements settlement
        JOIN football.matches fixture ON fixture.id=settlement.match_id
        JOIN football.teams home ON home.id=fixture.home_team_id
        JOIN football.teams away ON away.id=fixture.away_team_id
        JOIN football.competitions competition ON competition.id=settlement.competition_id
        JOIN model.versions version ON version.id=settlement.model_version_id
        JOIN model.parameter_sets parameter ON parameter.id=settlement.parameter_set_id
        LEFT JOIN LATERAL (
            SELECT count(*) AS item_count,
                   count(decision.id) FILTER (WHERE decision.verdict <> 'not_verifiable') AS scored_count
            FROM review.evidence_scoring_items item
            LEFT JOIN review.evidence_scoring_decisions decision ON decision.item_id=item.id
            WHERE item.settlement_id=settlement.id
        ) evidence_counts ON true
        LEFT JOIN LATERAL (
            SELECT run.status
            FROM analytics.postmatch_drift_runs run
            WHERE run.competition_id=settlement.competition_id
              AND run.competition_profile_id=settlement.competition_profile_id
              AND run.model_version_id=settlement.model_version_id
              AND run.parameter_set_id=settlement.parameter_set_id
              AND run.horizon=settlement.horizon
            ORDER BY run.generated_at DESC, run.id DESC LIMIT 1
        ) drift ON true
        {suffix}
        "#
    )
}

fn settlement_from_row(
    row: &sqlx::postgres::PgRow,
) -> PersistenceResult<PostmatchSettlementRecord> {
    Ok(PostmatchSettlementRecord {
        id: row.try_get("id")?,
        match_id: row.try_get("match_id")?,
        match_review_id: row.try_get("match_review_id")?,
        model_run_id: row.try_get("model_run_id")?,
        feature_snapshot_id: row.try_get("feature_snapshot_id")?,
        competition_id: row.try_get("competition_id")?,
        competition_name: row.try_get("competition_name")?,
        competition_profile_id: row.try_get("competition_profile_id")?,
        model_version_id: row.try_get("model_version_id")?,
        model_version: row.try_get("model_version")?,
        parameter_set_id: row.try_get("parameter_set_id")?,
        parameter_version: row.try_get("parameter_version")?,
        rule_package_id: row.try_get("rule_package_id")?,
        horizon: row.try_get("horizon")?,
        match_key: row.try_get("match_key")?,
        home_team_name: row.try_get("home_team_name")?,
        away_team_name: row.try_get("away_team_name")?,
        home_goals_90: row.try_get("home_goals_90")?,
        away_goals_90: row.try_get("away_goals_90")?,
        result_finalized_at: row.try_get("result_finalized_at")?,
        result_fingerprint: row.try_get("result_fingerprint")?,
        settlement_key: row.try_get("settlement_key")?,
        settlement_version: row.try_get("settlement_version")?,
        status: row.try_get("status")?,
        evidence_item_count: non_negative_u64(row.try_get("evidence_item_count")?, "证据队列数量")?,
        scored_evidence_count: non_negative_u64(
            row.try_get("scored_evidence_count")?,
            "已评分证据数量",
        )?,
        drift_status: row.try_get("drift_status")?,
        settled_by: row.try_get("settled_by")?,
        settlement_note: row.try_get("settlement_note")?,
        metadata: row.try_get("metadata")?,
        settled_at: row.try_get("settled_at")?,
    })
}
