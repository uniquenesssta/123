use crate::{PersistenceResult, PostgresStore};
use football_domain::{
    SpreadsheetExportData, SpreadsheetExternalIdRow, SpreadsheetPlayerAbilityRow,
    SpreadsheetPlayerAvailabilityRow, SpreadsheetPlayerDynamicTagRow, SpreadsheetPlayerNameRow,
    SpreadsheetPlayerPositionRow, SpreadsheetPlayerRow, SpreadsheetPlayerTeamPeriodRow,
    SpreadsheetTeamRow,
};
use sqlx::Row;

impl PostgresStore {
    pub async fn spreadsheet_export_data(&self) -> PersistenceResult<SpreadsheetExportData> {
        let teams = sqlx::query(
            "SELECT id, canonical_name, country_code, is_active FROM football.teams ORDER BY normalized_name, id",
        )
        .fetch_all(&self.pool)
        .await?
        .iter()
        .map(|row| Ok(SpreadsheetTeamRow {
            team_id: row.try_get("id")?,
            canonical_name: row.try_get("canonical_name")?,
            country_code: row.try_get("country_code")?,
            is_active: row.try_get("is_active")?,
        }))
        .collect::<PersistenceResult<Vec<_>>>()?;
        let players = sqlx::query(
            r#"
            SELECT id, canonical_name, date_of_birth, nationality_code,
                   preferred_foot, height_cm, status
            FROM football.players ORDER BY normalized_name, id
            "#,
        )
        .fetch_all(&self.pool)
        .await?
        .iter()
        .map(|row| {
            Ok(SpreadsheetPlayerRow {
                player_id: row.try_get("id")?,
                canonical_name: row.try_get("canonical_name")?,
                date_of_birth: row.try_get("date_of_birth")?,
                nationality_code: row.try_get("nationality_code")?,
                preferred_foot: row.try_get("preferred_foot")?,
                height_cm: row.try_get("height_cm")?,
                status: row.try_get("status")?,
            })
        })
        .collect::<PersistenceResult<Vec<_>>>()?;
        let names = sqlx::query(
            r#"
            SELECT name.player_id, player.canonical_name AS player_name,
                   player.date_of_birth AS player_birth_date, name.name,
                   name.language_code, name.is_primary, name.valid_from, name.valid_to
            FROM football.player_names name
            JOIN football.players player ON player.id = name.player_id
            ORDER BY player.normalized_name, name.is_primary DESC, name.name
            "#,
        )
        .fetch_all(&self.pool)
        .await?
        .iter()
        .map(|row| {
            Ok(SpreadsheetPlayerNameRow {
                player_id: row.try_get("player_id")?,
                player_name: row.try_get("player_name")?,
                player_birth_date: row.try_get("player_birth_date")?,
                name: row.try_get("name")?,
                language_code: row.try_get("language_code")?,
                is_primary: row.try_get("is_primary")?,
                valid_from: row.try_get("valid_from")?,
                valid_to: row.try_get("valid_to")?,
            })
        })
        .collect::<PersistenceResult<Vec<_>>>()?;
        let positions = sqlx::query(
            r#"
            SELECT position.player_id, player.canonical_name AS player_name,
                   player.date_of_birth AS player_birth_date, position.position_code,
                   position.proficiency, position.default_role_code, position.is_primary,
                   position.valid_from, position.valid_to
            FROM football.player_positions position
            JOIN football.players player ON player.id = position.player_id
            ORDER BY player.normalized_name, position.is_primary DESC, position.position_code
            "#,
        )
        .fetch_all(&self.pool)
        .await?
        .iter()
        .map(|row| {
            Ok(SpreadsheetPlayerPositionRow {
                player_id: row.try_get("player_id")?,
                player_name: row.try_get("player_name")?,
                player_birth_date: row.try_get("player_birth_date")?,
                position_code: row.try_get("position_code")?,
                proficiency: row.try_get("proficiency")?,
                default_role_code: row.try_get("default_role_code")?,
                is_primary: row.try_get("is_primary")?,
                valid_from: row.try_get("valid_from")?,
                valid_to: row.try_get("valid_to")?,
            })
        })
        .collect::<PersistenceResult<Vec<_>>>()?;
        let team_periods = sqlx::query(
            r#"
            SELECT period.player_id, player.canonical_name AS player_name,
                   player.date_of_birth AS player_birth_date, period.team_id,
                   team.canonical_name AS team_name, period.season_id, period.squad_number,
                   period.valid_from, period.valid_to, period.registration_status
            FROM football.player_team_periods period
            JOIN football.players player ON player.id = period.player_id
            JOIN football.teams team ON team.id = period.team_id
            ORDER BY player.normalized_name, period.valid_from DESC
            "#,
        )
        .fetch_all(&self.pool)
        .await?
        .iter()
        .map(|row| {
            Ok(SpreadsheetPlayerTeamPeriodRow {
                player_id: row.try_get("player_id")?,
                player_name: row.try_get("player_name")?,
                player_birth_date: row.try_get("player_birth_date")?,
                team_id: row.try_get("team_id")?,
                team_name: row.try_get("team_name")?,
                season_id: row.try_get("season_id")?,
                squad_number: row.try_get("squad_number")?,
                valid_from: row.try_get("valid_from")?,
                valid_to: row.try_get("valid_to")?,
                registration_status: row.try_get("registration_status")?,
            })
        })
        .collect::<PersistenceResult<Vec<_>>>()?;
        let abilities = sqlx::query(
            r#"
            SELECT observation.player_id, player.canonical_name AS player_name,
                   player.date_of_birth AS player_birth_date, observation.dimension_code,
                   observation.context_type, observation.context_id, observation.value,
                   observation.confidence, observation.sample_size, observation.observed_at,
                   observation.effective_from, observation.effective_to, observation.calculation_version
            FROM feature.player_ability_observations observation
            JOIN football.players player ON player.id = observation.player_id
            ORDER BY player.normalized_name, observation.dimension_code, observation.observed_at DESC
            "#,
        )
        .fetch_all(&self.pool)
        .await?
        .iter()
        .map(|row| Ok(SpreadsheetPlayerAbilityRow {
            player_id: row.try_get("player_id")?, player_name: row.try_get("player_name")?,
            player_birth_date: row.try_get("player_birth_date")?, dimension_code: row.try_get("dimension_code")?,
            context_type: row.try_get("context_type")?, context_id: row.try_get("context_id")?,
            value: row.try_get("value")?, confidence: row.try_get("confidence")?,
            sample_size: row.try_get("sample_size")?, observed_at: row.try_get("observed_at")?,
            effective_from: row.try_get("effective_from")?, effective_to: row.try_get("effective_to")?,
            calculation_version: row.try_get("calculation_version")?,
        }))
        .collect::<PersistenceResult<Vec<_>>>()?;
        let availability = sqlx::query(
            r#"
            SELECT availability.player_id, player.canonical_name AS player_name,
                   player.date_of_birth AS player_birth_date, availability.team_id,
                   team.canonical_name AS team_name, availability.competition_id,
                   availability.status, availability.reason, availability.confidence,
                   availability.valid_from, availability.valid_to
            FROM football.player_availability availability
            JOIN football.players player ON player.id = availability.player_id
            LEFT JOIN football.teams team ON team.id = availability.team_id
            ORDER BY player.normalized_name, availability.valid_from DESC
            "#,
        )
        .fetch_all(&self.pool)
        .await?
        .iter()
        .map(|row| {
            Ok(SpreadsheetPlayerAvailabilityRow {
                player_id: row.try_get("player_id")?,
                player_name: row.try_get("player_name")?,
                player_birth_date: row.try_get("player_birth_date")?,
                team_id: row.try_get("team_id")?,
                team_name: row.try_get("team_name")?,
                competition_id: row.try_get("competition_id")?,
                status: row.try_get("status")?,
                reason: row.try_get("reason")?,
                confidence: row.try_get("confidence")?,
                valid_from: row.try_get("valid_from")?,
                valid_to: row.try_get("valid_to")?,
            })
        })
        .collect::<PersistenceResult<Vec<_>>>()?;
        let dynamic_tags = sqlx::query(
            r#"
            SELECT tag.player_id, player.canonical_name AS player_name,
                   player.date_of_birth AS player_birth_date, tag.tag_code,
                   tag.value, tag.label, tag.confidence, tag.observed_at,
                   tag.valid_from, tag.valid_to, tag.competition_id,
                   tag.position_code, tag.opponent_team_id, tag.sample_size,
                   tag.source_type, tag.calculation_version
            FROM feature.player_dynamic_tags tag
            JOIN football.players player ON player.id = tag.player_id
            ORDER BY player.normalized_name, tag.tag_code, tag.observed_at DESC
            "#,
        )
        .fetch_all(&self.pool)
        .await?
        .iter()
        .map(|row| {
            Ok(SpreadsheetPlayerDynamicTagRow {
                player_id: row.try_get("player_id")?,
                player_name: row.try_get("player_name")?,
                player_birth_date: row.try_get("player_birth_date")?,
                tag_code: row.try_get("tag_code")?,
                value: row.try_get("value")?,
                label: row.try_get("label")?,
                confidence: row.try_get("confidence")?,
                observed_at: row.try_get("observed_at")?,
                valid_from: row.try_get("valid_from")?,
                valid_to: row.try_get("valid_to")?,
                competition_id: row.try_get("competition_id")?,
                position_code: row.try_get("position_code")?,
                opponent_team_id: row.try_get("opponent_team_id")?,
                sample_size: row.try_get("sample_size")?,
                source_type: row.try_get("source_type")?,
                calculation_version: row.try_get("calculation_version")?,
            })
        })
        .collect::<PersistenceResult<Vec<_>>>()?;
        let external_ids = sqlx::query(
            r#"
            SELECT provider.code AS provider_code, external.entity_type, external.entity_id,
                   COALESCE(player.canonical_name, team.canonical_name, external.entity_id::text) AS entity_name,
                   external.external_id
            FROM football.external_entity_ids external
            JOIN catalog.data_providers provider ON provider.id = external.provider_id
            LEFT JOIN football.players player ON external.entity_type = 'player' AND player.id = external.entity_id
            LEFT JOIN football.teams team ON external.entity_type = 'team' AND team.id = external.entity_id
            WHERE external.entity_type IN ('player', 'team')
            ORDER BY provider.code, external.entity_type, entity_name
            "#,
        )
        .fetch_all(&self.pool)
        .await?
        .iter()
        .map(|row| Ok(SpreadsheetExternalIdRow {
            provider_code: row.try_get("provider_code")?, entity_type: row.try_get("entity_type")?,
            entity_id: row.try_get("entity_id")?, entity_name: row.try_get("entity_name")?,
            external_id: row.try_get("external_id")?,
        }))
        .collect::<PersistenceResult<Vec<_>>>()?;
        Ok(SpreadsheetExportData {
            teams,
            players,
            names,
            positions,
            team_periods,
            abilities,
            availability,
            dynamic_tags,
            external_ids,
        })
    }
}
