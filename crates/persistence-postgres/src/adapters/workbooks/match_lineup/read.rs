use super::values::availability_from_str;
use crate::{PersistenceResult, PostgresStore};
use chrono::{DateTime, Utc};
use football_domain::{CoachListQuery, MatchLineupExportData, MatchLineupPlayerReference};
use sqlx::Row;
use uuid::Uuid;

impl PostgresStore {
    pub async fn match_lineup_export_data(
        &self,
        match_id: Option<Uuid>,
    ) -> PersistenceResult<MatchLineupExportData> {
        let selected_match = match match_id {
            Some(id) => Some(self.read_match(id).await?),
            None => None,
        };
        let lineups = if let Some(id) = match_id {
            self.hydrated_active_lineups(id).await?
        } else {
            Vec::new()
        };
        let competitions = self.list_competitions().await?;
        let references = self.player_catalog_reference_data().await?;
        let teams = references.teams.clone();
        let formations = references.formations.clone();
        let coaches = self.list_coaches(&CoachListQuery::default()).await?;
        let reference_time = selected_match
            .as_ref()
            .map(|item| item.kickoff_time)
            .unwrap_or_else(Utc::now);
        let players = if match_id.is_some() {
            self.match_player_references(match_id, reference_time)
                .await?
        } else {
            Vec::new()
        };
        let dynamic_tags = if match_id.is_some() {
            let mut tags = Vec::new();
            for player in &players {
                tags.extend(
                    self.list_player_dynamic_tags(player.player_id, reference_time)
                        .await?,
                );
            }
            tags
        } else {
            Vec::new()
        };
        Ok(MatchLineupExportData {
            selected_match,
            lineups,
            competitions,
            teams,
            formations,
            coaches,
            positions: references.positions,
            players,
            dynamic_tag_definitions: references.dynamic_tag_definitions,
            dynamic_tags,
        })
    }

    pub(super) async fn hydrated_active_lineups(
        &self,
        match_id: Uuid,
    ) -> PersistenceResult<Vec<football_domain::LineupRecord>> {
        let summaries = self.list_lineups(Some(match_id), 200).await?;
        let mut lineups = Vec::new();
        for summary in summaries.into_iter().filter(|item| item.status == "active") {
            lineups.push(self.read_lineup(summary.id).await?);
        }
        Ok(lineups)
    }

    async fn match_player_references(
        &self,
        match_id: Option<Uuid>,
        as_of: DateTime<Utc>,
    ) -> PersistenceResult<Vec<MatchLineupPlayerReference>> {
        let rows = sqlx::query(
            r#"
            SELECT DISTINCT player.id AS player_id, player.canonical_name, player.date_of_birth,
                   current_team.team_id AS current_team_id, team.canonical_name AS current_team_name,
                   primary_position.position_code AS primary_position_code,
                   primary_position.default_role_code AS primary_role_code,
                   availability.status AS availability_status,
                   profile.average_value AS ability_average,
                   profile.average_confidence AS ability_confidence
            FROM football.players player
            LEFT JOIN LATERAL (
                SELECT period.team_id FROM football.player_team_periods period
                WHERE period.player_id=player.id AND period.valid_from<=$2::date
                  AND (period.valid_to IS NULL OR period.valid_to>=$2::date)
                ORDER BY period.valid_from DESC LIMIT 1
            ) current_team ON true
            LEFT JOIN football.teams team ON team.id=current_team.team_id
            LEFT JOIN LATERAL (
                SELECT position_code, default_role_code FROM football.player_positions position
                WHERE position.player_id=player.id AND position.is_primary
                  AND (position.valid_from IS NULL OR position.valid_from<=$2::date)
                  AND (position.valid_to IS NULL OR position.valid_to>=$2::date)
                ORDER BY position.valid_from DESC NULLS LAST LIMIT 1
            ) primary_position ON true
            LEFT JOIN LATERAL (
                SELECT status FROM football.player_availability item
                WHERE item.player_id=player.id AND item.valid_from<=$2
                  AND (item.valid_to IS NULL OR item.valid_to>=$2)
                ORDER BY item.valid_from DESC LIMIT 1
            ) availability ON true
            LEFT JOIN feature.player_ability_profiles profile ON profile.player_id=player.id
            WHERE $1::uuid IS NULL OR EXISTS(
                SELECT 1
                FROM football.matches fixture
                JOIN football.player_team_periods period
                  ON period.team_id IN (fixture.home_team_id, fixture.away_team_id)
                 AND period.player_id=player.id
                 AND period.valid_from <= $2::date
                 AND (period.valid_to IS NULL OR period.valid_to >= $2::date)
                WHERE fixture.id=$1
            ) OR EXISTS(
                SELECT 1 FROM football.lineups lineup
                JOIN football.lineup_players lp ON lp.lineup_id=lineup.id
                WHERE lineup.match_id=$1 AND lp.player_id=player.id
            )
            ORDER BY player.canonical_name
            LIMIT 5000
            "#,
        ).bind(match_id).bind(as_of).fetch_all(&self.pool).await?;
        rows.iter()
            .map(|row| {
                Ok(MatchLineupPlayerReference {
                    player_id: row.try_get("player_id")?,
                    canonical_name: row.try_get("canonical_name")?,
                    date_of_birth: row.try_get("date_of_birth")?,
                    current_team_id: row.try_get("current_team_id")?,
                    current_team_name: row.try_get("current_team_name")?,
                    primary_position_code: row.try_get("primary_position_code")?,
                    primary_role_code: row.try_get("primary_role_code")?,
                    availability_status: row
                        .try_get::<Option<String>, _>("availability_status")?
                        .map(|value| availability_from_str(&value))
                        .transpose()?,
                    ability_average: row.try_get("ability_average")?,
                    ability_confidence: row.try_get("ability_confidence")?,
                })
            })
            .collect()
    }
}
