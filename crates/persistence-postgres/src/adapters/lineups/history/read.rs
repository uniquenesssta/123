use super::mapping::{lineup_player_from_row, lineup_record_from_row};
use crate::{PersistenceResult, PostgresStore};
use football_domain::{LineupRecord, TeamMatchLineupHistoryItem};
use sqlx::Row;
use uuid::Uuid;

impl PostgresStore {
    pub async fn list_lineups(
        &self,
        match_id: Option<Uuid>,
        limit: u32,
    ) -> PersistenceResult<Vec<LineupRecord>> {
        let rows = sqlx::query(
            r#"
            SELECT
                lineup.id, lineup.match_id, fixture.external_key AS match_key,
                lineup.team_id, team.canonical_name AS team_name,
                lineup.lineup_type, lineup.snapshot_type,
                lineup.formation, lineup.formation_id,
                formation.code AS formation_code, formation.name AS formation_name,
                lineup.coach_id, coach.canonical_name AS coach_name,
                lineup.captured_at, lineup.status, lineup.quality_score,
                lineup.source_urls, lineup.supersedes_lineup_id,
                lineup.model_validation_status, lineup.model_eligible,
                lineup.validation_errors, lineup.validation_warnings,
                count(player.player_id) AS player_count,
                count(player.player_id) FILTER (WHERE player.is_starter) AS starter_count
            FROM football.lineups lineup
            JOIN football.matches fixture ON fixture.id = lineup.match_id
            JOIN football.teams team ON team.id = lineup.team_id
            LEFT JOIN football.formations formation ON formation.id = lineup.formation_id
            LEFT JOIN football.coaches coach ON coach.id = lineup.coach_id
            LEFT JOIN football.lineup_players player ON player.lineup_id = lineup.id
            WHERE ($1::uuid IS NULL OR lineup.match_id = $1)
              AND lineup.history_hidden_at IS NULL
            GROUP BY lineup.id, fixture.external_key, team.canonical_name,
                     formation.code, formation.name, coach.canonical_name
            ORDER BY lineup.captured_at DESC, lineup.id DESC
            LIMIT $2
            "#,
        )
        .bind(match_id)
        .bind(i64::from(limit.clamp(1, 200)))
        .fetch_all(&self.pool)
        .await?;
        let mut result = Vec::with_capacity(rows.len());
        for row in rows {
            result.push(lineup_record_from_row(&row, Vec::new())?);
        }
        Ok(result)
    }

    pub async fn read_lineup(&self, lineup_id: Uuid) -> PersistenceResult<LineupRecord> {
        let row = sqlx::query(
            r#"
            SELECT
                lineup.id, lineup.match_id, fixture.external_key AS match_key,
                lineup.team_id, team.canonical_name AS team_name,
                lineup.lineup_type, lineup.snapshot_type,
                lineup.formation, lineup.formation_id,
                formation.code AS formation_code, formation.name AS formation_name,
                lineup.coach_id, coach.canonical_name AS coach_name,
                lineup.captured_at, lineup.status, lineup.quality_score,
                lineup.source_urls, lineup.supersedes_lineup_id,
                lineup.model_validation_status, lineup.model_eligible,
                lineup.validation_errors, lineup.validation_warnings,
                count(player.player_id) AS player_count,
                count(player.player_id) FILTER (WHERE player.is_starter) AS starter_count
            FROM football.lineups lineup
            JOIN football.matches fixture ON fixture.id = lineup.match_id
            JOIN football.teams team ON team.id = lineup.team_id
            LEFT JOIN football.formations formation ON formation.id = lineup.formation_id
            LEFT JOIN football.coaches coach ON coach.id = lineup.coach_id
            LEFT JOIN football.lineup_players player ON player.lineup_id = lineup.id
            WHERE lineup.id = $1
            GROUP BY lineup.id, fixture.external_key, team.canonical_name,
                     formation.code, formation.name, coach.canonical_name
            "#,
        )
        .bind(lineup_id)
        .fetch_one(&self.pool)
        .await?;
        let player_rows = sqlx::query(
            r#"
            SELECT player.player_id, football_player.canonical_name AS player_name,
                   player.position_code,
                   COALESCE(NULLIF(btrim(player.role_code), ''), inherited_role.default_role_code)
                       AS role_code,
                   CASE
                     WHEN player.metadata->>'role_origin' IN (
                       'lineup_override', 'player_position_default', 'missing'
                     ) THEN player.metadata->>'role_origin'
                     WHEN NULLIF(btrim(player.role_code), '') IS NOT NULL THEN 'lineup_override'
                     WHEN inherited_role.default_role_code IS NOT NULL THEN 'player_position_default'
                     ELSE 'missing'
                   END AS role_origin,
                   CASE
                     WHEN player.metadata->>'role_origin' = 'player_position_default'
                       THEN COALESCE(
                         NULLIF(btrim(player.metadata->>'role_source_position_code'), ''),
                         inherited_role.position_code
                       )
                     WHEN player.metadata->>'role_origin' IN ('lineup_override', 'missing')
                       THEN NULL
                     WHEN NULLIF(btrim(player.role_code), '') IS NOT NULL THEN NULL
                     WHEN inherited_role.default_role_code IS NOT NULL
                       THEN inherited_role.position_code
                     ELSE NULL
                   END AS role_source_position_code,
                   player.is_starter,
                   player.shirt_number, player.expected_minutes, player.actual_minutes,
                   player.sequence_no, player.bench_order, player.availability_status,
                   player.starting_probability, player.membership_override,
                   player.source_urls, player.validation_warning
            FROM football.lineup_players player
            JOIN football.lineups lineup ON lineup.id = player.lineup_id
            JOIN football.players football_player ON football_player.id = player.player_id
            LEFT JOIN LATERAL (
                SELECT position.default_role_code, position.position_code
                FROM football.player_positions position
                WHERE position.player_id = player.player_id
                  AND position.default_role_code IS NOT NULL
                  AND btrim(position.default_role_code) <> ''
                  AND (position.valid_from IS NULL OR position.valid_from <= lineup.captured_at::date)
                  AND (position.valid_to IS NULL OR position.valid_to >= lineup.captured_at::date)
                ORDER BY
                  CASE
                    WHEN player.position_code IS NOT NULL
                     AND upper(position.position_code) = upper(player.position_code) THEN 0
                    WHEN position.is_primary THEN 1
                    ELSE 2
                  END,
                  position.proficiency DESC,
                  position.valid_from DESC NULLS LAST,
                  position.id DESC
                LIMIT 1
            ) inherited_role ON true
            WHERE player.lineup_id = $1
            ORDER BY player.is_starter DESC, player.sequence_no,
                     player.bench_order NULLS LAST, football_player.normalized_name
            "#,
        )
        .bind(lineup_id)
        .fetch_all(&self.pool)
        .await?;
        let players = player_rows
            .iter()
            .map(lineup_player_from_row)
            .collect::<PersistenceResult<Vec<_>>>()?;
        lineup_record_from_row(&row, players)
    }

    pub async fn list_team_match_lineups(
        &self,
        team_id: Uuid,
        limit: u32,
    ) -> PersistenceResult<Vec<TeamMatchLineupHistoryItem>> {
        let rows = sqlx::query(
            r#"
            SELECT lineup.id, fixture.id AS match_id, fixture.external_key AS match_key,
                   fixture.kickoff_time,
                   CASE WHEN fixture.home_team_id=$1 THEN fixture.away_team_id ELSE fixture.home_team_id END AS opponent_team_id,
                   CASE WHEN fixture.home_team_id=$1 THEN away.canonical_name ELSE home.canonical_name END AS opponent_team_name,
                   CASE WHEN fixture.home_team_id=$1 THEN 'home' ELSE 'away' END AS venue_side
            FROM football.lineups lineup
            JOIN football.matches fixture ON fixture.id=lineup.match_id
            JOIN football.teams home ON home.id=fixture.home_team_id
            JOIN football.teams away ON away.id=fixture.away_team_id
            WHERE lineup.team_id=$1
              AND lineup.history_hidden_at IS NULL
            ORDER BY fixture.kickoff_time DESC, lineup.captured_at DESC, lineup.id DESC
            LIMIT $2
            "#,
        )
        .bind(team_id)
        .bind(i64::from(limit.clamp(1, 200)))
        .fetch_all(&self.pool)
        .await?;
        let mut result = Vec::with_capacity(rows.len());
        for row in rows {
            let lineup_id: Uuid = row.try_get("id")?;
            result.push(TeamMatchLineupHistoryItem {
                match_id: row.try_get("match_id")?,
                match_key: row.try_get("match_key")?,
                opponent_team_id: row.try_get("opponent_team_id")?,
                opponent_team_name: row.try_get("opponent_team_name")?,
                venue_side: row.try_get("venue_side")?,
                kickoff_time: row.try_get("kickoff_time")?,
                lineup: self.read_lineup(lineup_id).await?,
            });
        }
        Ok(result)
    }
}
