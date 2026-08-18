use crate::{
    adapters::catalog::formations::constants::UNKNOWN_FORMATION_ID, PersistenceResult,
    PostgresStore,
};
use chrono::Utc;
use football_domain::{
    FormationDistributionQuery, FormationUsageEntryRecord, ResolvedFormationDistribution,
};
use sqlx::Row;
use uuid::Uuid;

impl PostgresStore {
    pub async fn resolve_formation_distribution(
        &self,
        query: &FormationDistributionQuery,
    ) -> PersistenceResult<ResolvedFormationDistribution> {
        let as_of = query.as_of.unwrap_or_else(Utc::now);
        let as_of_date = as_of.date_naive();
        let mut competition_id = query.competition_id;

        if let Some(match_id) = query.match_id {
            let lineup_row = sqlx::query(
                r#"
                    SELECT lineup.formation_id, formation.code, formation.name,
                           lineup.quality_score, lineup.lineup_type,
                           fixture.competition_id
                    FROM football.lineups lineup
                    JOIN football.matches fixture ON fixture.id = lineup.match_id
                    JOIN football.formations formation ON formation.id = lineup.formation_id
                    WHERE lineup.match_id = $1
                      AND lineup.team_id = $2
                      AND lineup.status = 'active'
                      AND lineup.lineup_type IN ('actual','confirmed')
                      AND lineup.captured_at <= $3
                    ORDER BY CASE lineup.lineup_type WHEN 'actual' THEN 0 ELSE 1 END,
                             lineup.captured_at DESC, lineup.id DESC
                    LIMIT 1
                    "#,
            )
            .bind(match_id)
            .bind(query.team_id)
            .bind(as_of)
            .fetch_optional(&self.pool)
            .await?;
            if let Some(row) = lineup_row {
                let formation_id: Uuid = row.try_get("formation_id")?;
                let lineup_type: String = row.try_get("lineup_type")?;
                let (source_level, source_label) = if lineup_type == "actual" {
                    ("actual_lineup", "当前比赛实际阵型")
                } else {
                    ("confirmed_lineup", "当前比赛确认阵型")
                };
                competition_id = competition_id.or(row.try_get("competition_id")?);
                return Ok(ResolvedFormationDistribution {
                    source_level: source_level.to_string(),
                    source_label: source_label.to_string(),
                    team_id: query.team_id,
                    coach_id: query.coach_id,
                    competition_id,
                    window_start: None,
                    window_end: None,
                    observed_matches: 1,
                    confidence: row
                        .try_get::<Option<f64>, _>("quality_score")?
                        .unwrap_or(1.0),
                    entries: vec![FormationUsageEntryRecord {
                        id: Uuid::nil(),
                        formation_id,
                        formation_code: row.try_get("code")?,
                        formation_name: row.try_get("name")?,
                        usage_count: 1,
                        raw_probability: 1.0,
                        smoothed_probability: 1.0,
                    }],
                });
            }
            if competition_id.is_none() {
                competition_id =
                    sqlx::query_scalar("SELECT competition_id FROM football.matches WHERE id=$1")
                        .bind(match_id)
                        .fetch_optional(&self.pool)
                        .await?
                        .flatten();
            }
        }

        let coach_id = if query.coach_id.is_some() {
            query.coach_id
        } else {
            sqlx::query_scalar(
                    r#"
                    SELECT coach_id
                    FROM football.team_coach_periods
                    WHERE team_id=$1
                      AND role IN ('head_coach','interim_head_coach','caretaker')
                      AND valid_from <= $2
                      AND (valid_to IS NULL OR valid_to >= $2)
                    ORDER BY CASE role WHEN 'head_coach' THEN 0 WHEN 'interim_head_coach' THEN 1 ELSE 2 END,
                             valid_from DESC, id DESC
                    LIMIT 1
                    "#,
                )
                .bind(query.team_id)
                .bind(as_of_date)
                .fetch_optional(&self.pool)
                .await?
        };

        let candidates = [
            (
                "team_coach",
                Some(query.team_id),
                coach_id,
                None,
                "球队 + 教练",
            ),
            ("team", Some(query.team_id), None, None, "球队"),
            ("coach", None, coach_id, None, "教练"),
            (
                "competition_default",
                None,
                None,
                competition_id,
                "赛事默认",
            ),
            ("system_default", None, None, None, "系统默认"),
        ];
        for (scope, team, coach, competition, label) in candidates {
            if (scope == "team_coach" || scope == "coach") && coach.is_none() {
                continue;
            }
            if scope == "competition_default" && competition.is_none() {
                continue;
            }
            if let Some(distribution) = self
                .read_latest_distribution(scope, team, coach, competition, as_of)
                .await?
            {
                return Ok(ResolvedFormationDistribution {
                    source_level: scope.to_string(),
                    source_label: label.to_string(),
                    team_id: query.team_id,
                    coach_id,
                    competition_id,
                    window_start: Some(distribution.window_start),
                    window_end: Some(distribution.window_end),
                    observed_matches: distribution.observed_matches,
                    confidence: distribution.confidence,
                    entries: distribution.entries,
                });
            }
        }

        let unknown = sqlx::query("SELECT id, code, name FROM football.formations WHERE id=$1")
            .bind(UNKNOWN_FORMATION_ID)
            .fetch_one(&self.pool)
            .await?;
        Ok(ResolvedFormationDistribution {
            source_level: "unknown".to_string(),
            source_label: "无可用观察，回退未知".to_string(),
            team_id: query.team_id,
            coach_id,
            competition_id,
            window_start: None,
            window_end: None,
            observed_matches: 0,
            confidence: 0.0,
            entries: vec![FormationUsageEntryRecord {
                id: Uuid::nil(),
                formation_id: unknown.try_get("id")?,
                formation_code: unknown.try_get("code")?,
                formation_name: unknown.try_get("name")?,
                usage_count: 0,
                raw_probability: 1.0,
                smoothed_probability: 1.0,
            }],
        })
    }
}
