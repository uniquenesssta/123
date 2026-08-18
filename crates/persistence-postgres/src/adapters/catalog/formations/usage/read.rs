use super::{grouping::group_distribution_rows, key::FormationUsageGroupKey};
use crate::{PersistenceResult, PostgresStore};
use chrono::{DateTime, Utc};
use football_domain::{FormationUsageDistributionRecord, FormationUsageListQuery};
use sqlx::Row;
use uuid::Uuid;

impl PostgresStore {
    pub async fn list_formation_usage_distributions(
        &self,
        query: &FormationUsageListQuery,
    ) -> PersistenceResult<Vec<FormationUsageDistributionRecord>> {
        let rows = sqlx::query(
                r#"
                WITH selected_groups AS (
                    SELECT DISTINCT
                        observation.scope_type, observation.team_id, observation.coach_id,
                        observation.competition_id, observation.window_start,
                        observation.window_end, observation.observed_at
                    FROM feature.formation_usage_observations observation
                    WHERE ($1::uuid IS NULL OR observation.team_id = $1)
                      AND ($2::uuid IS NULL OR observation.coach_id = $2)
                      AND ($3::uuid IS NULL OR observation.competition_id = $3)
                ), limited_groups AS (
                    SELECT *
                    FROM selected_groups
                    ORDER BY observed_at DESC, window_end DESC, scope_type
                    LIMIT $4
                )
                SELECT observation.id, observation.scope_type,
                       observation.team_id, team.canonical_name AS team_name,
                       observation.coach_id, coach.canonical_name AS coach_name,
                       observation.competition_id, competition.name AS competition_name,
                       observation.window_preset, observation.window_start, observation.window_end,
                       observation.observed_matches, observation.confidence,
                       observation.smoothing_alpha, observation.observed_at,
                       observation.formation_id, formation.code AS formation_code,
                       formation.name AS formation_name, observation.usage_count,
                       observation.raw_probability, observation.smoothed_probability
                FROM limited_groups selected
                JOIN feature.formation_usage_observations observation
                  ON observation.scope_type = selected.scope_type
                 AND observation.team_id IS NOT DISTINCT FROM selected.team_id
                 AND observation.coach_id IS NOT DISTINCT FROM selected.coach_id
                 AND observation.competition_id IS NOT DISTINCT FROM selected.competition_id
                 AND observation.window_start = selected.window_start
                 AND observation.window_end = selected.window_end
                 AND observation.observed_at = selected.observed_at
                JOIN football.formations formation ON formation.id = observation.formation_id
                LEFT JOIN football.teams team ON team.id = observation.team_id
                LEFT JOIN football.coaches coach ON coach.id = observation.coach_id
                LEFT JOIN football.competitions competition ON competition.id = observation.competition_id
                ORDER BY observation.observed_at DESC, observation.window_end DESC,
                         observation.scope_type, observation.smoothed_probability DESC,
                         formation.sort_order, formation.code
                "#,
            )
            .bind(query.team_id)
            .bind(query.coach_id)
            .bind(query.competition_id)
            .bind(i64::from(query.limit.clamp(1, 1000)))
            .fetch_all(&self.pool)
            .await?;
        group_distribution_rows(&rows)
    }
}

impl PostgresStore {
    pub(crate) async fn read_latest_distribution(
        &self,
        scope_type: &str,
        team_id: Option<Uuid>,
        coach_id: Option<Uuid>,
        competition_id: Option<Uuid>,
        as_of: DateTime<Utc>,
    ) -> PersistenceResult<Option<FormationUsageDistributionRecord>> {
        let group = sqlx::query(
            r#"
                SELECT window_start, window_end, observed_at
                FROM feature.formation_usage_observations
                WHERE scope_type=$1
                  AND team_id IS NOT DISTINCT FROM $2
                  AND coach_id IS NOT DISTINCT FROM $3
                  AND competition_id IS NOT DISTINCT FROM $4
                  AND window_start <= $5
                  AND window_end <= $5
                  AND observed_at <= $6
                ORDER BY window_end DESC, observed_at DESC
                LIMIT 1
                "#,
        )
        .bind(scope_type)
        .bind(team_id)
        .bind(coach_id)
        .bind(competition_id)
        .bind(as_of.date_naive())
        .bind(as_of)
        .fetch_optional(&self.pool)
        .await?;
        let Some(group) = group else {
            return Ok(None);
        };
        self.read_exact_distribution(&FormationUsageGroupKey {
            scope_type,
            team_id,
            coach_id,
            competition_id,
            window_start: group.try_get("window_start")?,
            window_end: group.try_get("window_end")?,
            observed_at: group.try_get("observed_at")?,
        })
        .await
    }
}

impl PostgresStore {
    pub(crate) async fn read_exact_distribution(
        &self,
        key: &FormationUsageGroupKey<'_>,
    ) -> PersistenceResult<Option<FormationUsageDistributionRecord>> {
        let rows = sqlx::query(
            r#"
                SELECT observation.id, observation.scope_type,
                       observation.team_id, team.canonical_name AS team_name,
                       observation.coach_id, coach.canonical_name AS coach_name,
                       observation.competition_id, competition.name AS competition_name,
                       observation.window_preset, observation.window_start, observation.window_end,
                       observation.observed_matches, observation.confidence,
                       observation.smoothing_alpha, observation.observed_at,
                       observation.formation_id, formation.code AS formation_code,
                       formation.name AS formation_name, observation.usage_count,
                       observation.raw_probability, observation.smoothed_probability
                FROM feature.formation_usage_observations observation
                JOIN football.formations formation ON formation.id=observation.formation_id
                LEFT JOIN football.teams team ON team.id=observation.team_id
                LEFT JOIN football.coaches coach ON coach.id=observation.coach_id
                LEFT JOIN football.competitions competition ON competition.id=observation.competition_id
                WHERE observation.scope_type=$1
                  AND observation.team_id IS NOT DISTINCT FROM $2
                  AND observation.coach_id IS NOT DISTINCT FROM $3
                  AND observation.competition_id IS NOT DISTINCT FROM $4
                  AND observation.window_start=$5
                  AND observation.window_end=$6
                  AND observation.observed_at=$7
                ORDER BY observation.smoothed_probability DESC, formation.sort_order, formation.code
                "#,
        )
        .bind(key.scope_type)
        .bind(key.team_id)
        .bind(key.coach_id)
        .bind(key.competition_id)
        .bind(key.window_start)
        .bind(key.window_end)
        .bind(key.observed_at)
        .fetch_all(&self.pool)
        .await?;
        Ok(group_distribution_rows(&rows)?.into_iter().next())
    }
}
