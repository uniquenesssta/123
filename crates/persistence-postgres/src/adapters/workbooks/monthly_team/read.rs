use crate::{PersistenceResult, PostgresStore};
use football_domain::{
    TeamAbilityObservationRow, TeamMonthlyCoachPeriodRow, TeamMonthlyCoachRow,
    TeamMonthlyFormationUsageRow, TeamMonthlyNameRow, TeamMonthlyTeamRow, TeamMonthlyWorkbookData,
    TeamTacticalObservationRow,
};
use sqlx::Row;

impl PostgresStore {
    pub async fn team_monthly_workbook_data(&self) -> PersistenceResult<TeamMonthlyWorkbookData> {
        let teams = sqlx::query(
            r#"
            SELECT team.id, team.canonical_name, team.country_code, team.is_active, team.metadata,
                   profile.short_name, COALESCE(profile.team_type, 'club') AS team_type,
                   profile.city, profile.founded_year, profile.stadium,
                   profile.updated_at AS profile_observed_at,
                   COALESCE(profile.data_confidence, 0.5) AS data_confidence,
                   profile.notes
            FROM football.teams team
            LEFT JOIN football.team_profiles profile ON profile.team_id = team.id
            ORDER BY team.canonical_name, team.id
            "#,
        )
        .fetch_all(&self.pool)
        .await?
        .iter()
        .map(|row| {
            Ok(TeamMonthlyTeamRow {
                team_id: row.try_get("id")?,
                official_name: row.try_get("canonical_name")?,
                short_name: row.try_get("short_name")?,
                team_type: row.try_get("team_type")?,
                country_code: row.try_get("country_code")?,
                city: row.try_get("city")?,
                founded_year: row.try_get("founded_year")?,
                stadium: row.try_get("stadium")?,
                is_active: row.try_get("is_active")?,
                profile_observed_at: row.try_get("profile_observed_at")?,
                data_confidence: row.try_get("data_confidence")?,
                notes: row.try_get("notes")?,
                metadata: row.try_get("metadata")?,
            })
        })
        .collect::<PersistenceResult<Vec<_>>>()?;

        let names = sqlx::query(
            r#"
            SELECT name.team_id, team.canonical_name AS official_name, name.name AS name_value,
                   name.language_code, name.valid_from, name.valid_to, name.metadata
            FROM football.team_names name
            JOIN football.teams team ON team.id = name.team_id
            ORDER BY team.canonical_name, name.name, name.id
            "#,
        )
        .fetch_all(&self.pool)
        .await?
        .iter()
        .map(|row| {
            Ok(TeamMonthlyNameRow {
                team_id: row.try_get("team_id")?,
                official_name: row.try_get("official_name")?,
                name_value: row.try_get("name_value")?,
                language_code: row.try_get("language_code")?,
                valid_from: row.try_get("valid_from")?,
                valid_to: row.try_get("valid_to")?,
                metadata: row.try_get("metadata")?,
            })
        })
        .collect::<PersistenceResult<Vec<_>>>()?;

        let coaches = sqlx::query(
            r#"
            SELECT id, canonical_name, nationality_code, status, metadata
            FROM football.coaches ORDER BY canonical_name, id
            "#,
        )
        .fetch_all(&self.pool)
        .await?
        .iter()
        .map(|row| {
            Ok(TeamMonthlyCoachRow {
                coach_id: row.try_get("id")?,
                official_name: row.try_get("canonical_name")?,
                nationality_code: row.try_get("nationality_code")?,
                status: row.try_get("status")?,
                metadata: row.try_get("metadata")?,
            })
        })
        .collect::<PersistenceResult<Vec<_>>>()?;

        let coach_periods = sqlx::query(
            r#"
            SELECT period.team_id, team.canonical_name AS team_name,
                   period.coach_id, coach.canonical_name AS coach_name,
                   period.role, period.valid_from, period.valid_to, period.is_interim,
                   period.confidence, period.metadata
            FROM football.team_coach_periods period
            JOIN football.teams team ON team.id = period.team_id
            JOIN football.coaches coach ON coach.id = period.coach_id
            ORDER BY team.canonical_name, period.valid_from DESC, period.id
            "#,
        )
        .fetch_all(&self.pool)
        .await?
        .iter()
        .map(|row| {
            Ok(TeamMonthlyCoachPeriodRow {
                team_id: row.try_get("team_id")?,
                team_name: row.try_get("team_name")?,
                coach_id: row.try_get("coach_id")?,
                coach_name: row.try_get("coach_name")?,
                role: row.try_get("role")?,
                valid_from: row.try_get("valid_from")?,
                valid_to: row.try_get("valid_to")?,
                is_interim: row.try_get("is_interim")?,
                confidence: row.try_get("confidence")?,
                metadata: row.try_get("metadata")?,
            })
        })
        .collect::<PersistenceResult<Vec<_>>>()?;

        let formation_usage = sqlx::query(
            r#"
            SELECT observation.scope_type, observation.team_id, team.canonical_name AS team_name,
                   observation.coach_id, coach.canonical_name AS coach_name,
                   observation.competition_id, observation.formation_id, formation.code AS formation_code,
                   observation.window_preset, observation.window_start, observation.window_end,
                   observation.observed_matches, observation.usage_count, observation.raw_probability,
                   observation.smoothed_probability, observation.confidence,
                   observation.smoothing_alpha AS alpha, observation.observed_at, observation.metadata
            FROM feature.formation_usage_observations observation
            JOIN football.formations formation ON formation.id = observation.formation_id
            LEFT JOIN football.teams team ON team.id = observation.team_id
            LEFT JOIN football.coaches coach ON coach.id = observation.coach_id
            ORDER BY observation.observed_at DESC, observation.scope_type, formation.sort_order
            "#,
        )
        .fetch_all(&self.pool)
        .await?
        .iter()
        .map(|row| {
            Ok(TeamMonthlyFormationUsageRow {
                scope_type: row.try_get("scope_type")?,
                team_id: row.try_get("team_id")?,
                team_name: row.try_get("team_name")?,
                coach_id: row.try_get("coach_id")?,
                coach_name: row.try_get("coach_name")?,
                competition_id: row.try_get("competition_id")?,
                formation_id: row.try_get("formation_id")?,
                formation_code: row.try_get("formation_code")?,
                window_preset: row.try_get("window_preset")?,
                window_start: row.try_get("window_start")?,
                window_end: row.try_get("window_end")?,
                observed_matches: row.try_get("observed_matches")?,
                usage_count: row.try_get("usage_count")?,
                raw_probability: row.try_get("raw_probability")?,
                smoothed_probability: row.try_get("smoothed_probability")?,
                confidence: row.try_get("confidence")?,
                alpha: row.try_get("alpha")?,
                observed_at: row.try_get("observed_at")?,
                metadata: row.try_get("metadata")?,
            })
        })
        .collect::<PersistenceResult<Vec<_>>>()?;

        let tactical_observations = sqlx::query(
            r#"
            SELECT observation.team_id, team.canonical_name AS team_name,
                   observation.coach_id, coach.canonical_name AS coach_name,
                   observation.window_start, observation.window_end,
                   observation.build_up_style, observation.progression_style,
                   observation.attacking_width, observation.pressing_intensity,
                   observation.defensive_block, observation.transition_speed,
                   observation.set_piece_tendency, observation.tactical_summary,
                   observation.confidence, observation.observed_at,
                   observation.metadata || jsonb_build_object(
                       'source_urls', observation.source_urls,
                       'verified_at', observation.verified_at
                   ) AS metadata
            FROM feature.team_tactical_observations observation
            JOIN football.teams team ON team.id = observation.team_id
            LEFT JOIN football.coaches coach ON coach.id = observation.coach_id
            ORDER BY observation.observed_at DESC, team.canonical_name
            "#,
        )
        .fetch_all(&self.pool)
        .await?
        .iter()
        .map(|row| {
            Ok(TeamTacticalObservationRow {
                team_id: row.try_get("team_id")?,
                team_name: row.try_get("team_name")?,
                coach_id: row.try_get("coach_id")?,
                coach_name: row.try_get("coach_name")?,
                window_start: row.try_get("window_start")?,
                window_end: row.try_get("window_end")?,
                build_up_style: row.try_get("build_up_style")?,
                progression_style: row.try_get("progression_style")?,
                attacking_width: row.try_get("attacking_width")?,
                pressing_intensity: row.try_get("pressing_intensity")?,
                defensive_block: row.try_get("defensive_block")?,
                transition_speed: row.try_get("transition_speed")?,
                set_piece_tendency: row.try_get("set_piece_tendency")?,
                tactical_summary: row.try_get("tactical_summary")?,
                confidence: row.try_get("confidence")?,
                observed_at: row.try_get("observed_at")?,
                metadata: row.try_get("metadata")?,
            })
        })
        .collect::<PersistenceResult<Vec<_>>>()?;

        let ability_observations = sqlx::query(
            r#"
            SELECT observation.team_id, team.canonical_name AS team_name,
                   observation.observed_at, observation.window_start, observation.window_end,
                   observation.attack_rating, observation.midfield_rating,
                   observation.defence_rating, observation.goalkeeper_rating,
                   observation.squad_depth_rating, observation.stability_rating,
                   observation.sample_size, observation.methodology, observation.confidence,
                   observation.metadata || jsonb_build_object(
                       'source_urls', observation.source_urls,
                       'verified_at', observation.verified_at
                   ) AS metadata
            FROM feature.team_ability_observations observation
            JOIN football.teams team ON team.id = observation.team_id
            ORDER BY observation.observed_at DESC, team.canonical_name
            "#,
        )
        .fetch_all(&self.pool)
        .await?
        .iter()
        .map(|row| {
            Ok(TeamAbilityObservationRow {
                team_id: row.try_get("team_id")?,
                team_name: row.try_get("team_name")?,
                observed_at: row.try_get("observed_at")?,
                window_start: row.try_get("window_start")?,
                window_end: row.try_get("window_end")?,
                attack_rating: row.try_get("attack_rating")?,
                midfield_rating: row.try_get("midfield_rating")?,
                defence_rating: row.try_get("defence_rating")?,
                goalkeeper_rating: row.try_get("goalkeeper_rating")?,
                squad_depth_rating: row.try_get("squad_depth_rating")?,
                stability_rating: row.try_get("stability_rating")?,
                sample_size: row.try_get("sample_size")?,
                methodology: row.try_get("methodology")?,
                confidence: row.try_get("confidence")?,
                metadata: row.try_get("metadata")?,
            })
        })
        .collect::<PersistenceResult<Vec<_>>>()?;

        let data_gaps = self.team_monthly_data_gaps().await?;
        Ok(TeamMonthlyWorkbookData {
            teams,
            names,
            coaches,
            coach_periods,
            formation_usage,
            tactical_observations,
            ability_observations,
            data_gaps,
        })
    }
}
