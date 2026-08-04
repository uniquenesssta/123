use crate::{
    name_search::{push_name_search, NameSearch, NameSearchColumns},
    role_resolution::{
        metadata_with_role_resolution, resolve_default_tactical_role_in_tx, resolve_tactical_role,
    },
    PersistenceError, PersistenceResult, PostgresStore,
};
use chrono::{Datelike, NaiveDate, Utc};
use football_domain::{
    AbilityDimensionRecord, AvailabilityStatus, DataProviderDraft, DataProviderRecord,
    ExternalEntityIdDraft, ExternalEntityIdRecord, LineupDraft, LineupHistoryRemovalResult,
    LineupPairDraft, LineupPairRecord, LineupPlayerRecord, LineupRecord, LineupType, MatchDraft,
    MatchRecord, MatchStatus, PlayerAbilityObservationDraft, PlayerAbilityObservationRecord,
    PlayerAbilityProfile, PlayerAvailabilityDraft, PlayerAvailabilityRecord,
    PlayerCatalogReferenceData, PlayerDetail, PlayerDraft, PlayerListItem, PlayerListPage,
    PlayerListQuery, PlayerNameDraft, PlayerNameRecord, PlayerPositionDraft, PlayerPositionRecord,
    PlayerRecord, PlayerStatus, PlayerTeamPeriodDraft, PlayerTeamPeriodRecord, PositionReference,
    PreferredFoot, SeasonTeamMembershipOption, TeamDraft, TeamOption, TeamRecord,
};
use serde_json::{json, Value};
use sqlx::{Postgres, QueryBuilder, Row, Transaction};
use std::collections::HashSet;
use uuid::Uuid;

impl PostgresStore {
    pub async fn create_team(&self, draft: &TeamDraft) -> PersistenceResult<TeamRecord> {
        let canonical_name = draft.canonical_name.trim();
        if canonical_name.is_empty() {
            return Err(PersistenceError::InvalidState(
                "球队名称不能为空".to_string(),
            ));
        }
        let normalized_name = normalize_name(canonical_name);
        let id = Uuid::new_v4();
        let row = sqlx::query(
            r#"
            INSERT INTO football.teams (
                id, canonical_name, normalized_name, country_code, metadata
            ) VALUES ($1, $2, $3, $4, $5)
            RETURNING id, canonical_name, normalized_name, country_code, is_active, created_at
            "#,
        )
        .bind(id)
        .bind(canonical_name)
        .bind(&normalized_name)
        .bind(
            draft
                .country_code
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty()),
        )
        .bind(&draft.metadata)
        .fetch_one(&self.pool)
        .await?;
        self.audit(
            "team_created",
            "team",
            id.to_string(),
            json!({"canonical_name": canonical_name}),
        )
        .await?;
        team_record_from_row(&row)
    }

    pub async fn list_team_options(
        &self,
        search: Option<&str>,
        limit: u32,
    ) -> PersistenceResult<Vec<TeamOption>> {
        let safe_limit = limit.clamp(1, 500) as i64;
        let mut builder: QueryBuilder<Postgres> = QueryBuilder::new(
            r#"
            SELECT football.teams.id, football.teams.canonical_name, football.teams.country_code,
                   COALESCE(profile.team_type, 'other') AS team_type
            FROM football.teams
            LEFT JOIN football.team_profiles profile ON profile.team_id = football.teams.id
            WHERE football.teams.is_active
            "#,
        );
        if let Some(search) = NameSearch::parse(search) {
            push_name_search(
                &mut builder,
                &search,
                NameSearchColumns {
                    primary_normalized: "football.teams.normalized_name",
                    primary_display: "football.teams.canonical_name",
                    alias_table: "football.team_names",
                    alias_owner: "alias.team_id",
                    owner_id: "football.teams.id",
                    alias_normalized: "alias.normalized_name",
                    alias_display: "alias.name",
                },
            );
        }
        builder.push(" ORDER BY football.teams.normalized_name, football.teams.id LIMIT ");
        builder.push_bind(safe_limit);
        builder
            .build()
            .fetch_all(&self.pool)
            .await?
            .iter()
            .map(team_option_from_row)
            .collect()
    }

    pub async fn create_data_provider(
        &self,
        draft: &DataProviderDraft,
    ) -> PersistenceResult<DataProviderRecord> {
        let code = draft.code.trim().to_lowercase();
        let name = draft.name.trim();
        if code.is_empty() || name.is_empty() || draft.provider_type.trim().is_empty() {
            return Err(PersistenceError::InvalidState(
                "数据源代码、名称和类型不能为空".to_string(),
            ));
        }
        let generated_id = Uuid::new_v4();
        let row = sqlx::query(
            r#"
            INSERT INTO catalog.data_providers (
                id, code, name, provider_type, base_url, metadata
            ) VALUES ($1, $2, $3, $4, $5, $6)
            ON CONFLICT (code) DO UPDATE SET
                name = EXCLUDED.name,
                provider_type = EXCLUDED.provider_type,
                base_url = EXCLUDED.base_url,
                metadata = catalog.data_providers.metadata || EXCLUDED.metadata,
                is_active = true,
                updated_at = now()
            RETURNING id, code, name, provider_type, base_url, is_active
            "#,
        )
        .bind(generated_id)
        .bind(&code)
        .bind(name)
        .bind(draft.provider_type.trim())
        .bind(
            draft
                .base_url
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty()),
        )
        .bind(&draft.metadata)
        .fetch_one(&self.pool)
        .await?;
        data_provider_from_row(&row)
    }

    pub async fn list_data_providers(&self) -> PersistenceResult<Vec<DataProviderRecord>> {
        let rows = sqlx::query(
            r#"
            SELECT id, code, name, provider_type, base_url, is_active
            FROM catalog.data_providers
            WHERE is_active
            ORDER BY name, code
            "#,
        )
        .fetch_all(&self.pool)
        .await?;
        rows.iter().map(data_provider_from_row).collect()
    }

    pub async fn create_player(&self, draft: &PlayerDraft) -> PersistenceResult<PlayerRecord> {
        let canonical_name = draft.canonical_name.trim();
        if canonical_name.is_empty() {
            return Err(PersistenceError::InvalidState(
                "球员姓名不能为空".to_string(),
            ));
        }
        if let Some(height) = draft.height_cm {
            if !(120..=230).contains(&height) {
                return Err(PersistenceError::InvalidState(
                    "球员身高必须位于 120–230 cm".to_string(),
                ));
            }
        }
        let normalized_name = normalize_name(canonical_name);
        let id = Uuid::new_v4();
        let mut tx = self.pool.begin().await?;
        let row = sqlx::query(
            r#"
            INSERT INTO football.players (
                id, canonical_name, normalized_name, date_of_birth,
                nationality_code, preferred_foot, height_cm, status, metadata
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            RETURNING
                id, canonical_name, normalized_name, date_of_birth,
                nationality_code, preferred_foot, height_cm, status, created_at
            "#,
        )
        .bind(id)
        .bind(canonical_name)
        .bind(&normalized_name)
        .bind(draft.date_of_birth)
        .bind(
            draft
                .nationality_code
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty()),
        )
        .bind(draft.preferred_foot.as_str())
        .bind(draft.height_cm)
        .bind(draft.status.as_str())
        .bind(&draft.metadata)
        .fetch_one(&mut *tx)
        .await?;

        sqlx::query(
            r#"
            INSERT INTO football.player_names (
                id, player_id, name, normalized_name, is_primary
            ) VALUES ($1, $2, $3, $4, true)
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(id)
        .bind(canonical_name)
        .bind(&normalized_name)
        .execute(&mut *tx)
        .await?;
        audit_in_tx(
            &mut tx,
            "player_created",
            "player",
            id.to_string(),
            json!({"canonical_name": canonical_name}),
        )
        .await?;
        tx.commit().await?;
        player_record_from_row(&row)
    }

    pub async fn update_player(
        &self,
        player_id: Uuid,
        draft: &PlayerDraft,
    ) -> PersistenceResult<PlayerRecord> {
        let canonical_name = draft.canonical_name.trim();
        if canonical_name.is_empty() {
            return Err(PersistenceError::InvalidState(
                "球员姓名不能为空".to_string(),
            ));
        }
        if let Some(height) = draft.height_cm {
            if !(120..=230).contains(&height) {
                return Err(PersistenceError::InvalidState(
                    "球员身高必须位于 120–230 cm".to_string(),
                ));
            }
        }
        let normalized_name = normalize_name(canonical_name);
        let mut tx = self.pool.begin().await?;
        let previous_normalized_name: String = sqlx::query_scalar(
            "SELECT normalized_name FROM football.players WHERE id = $1 FOR UPDATE",
        )
        .bind(player_id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| PersistenceError::InvalidState("球员不存在".to_string()))?;
        let row = sqlx::query(
            r#"
            UPDATE football.players SET
                canonical_name = $2,
                normalized_name = $3,
                date_of_birth = $4,
                nationality_code = $5,
                preferred_foot = $6,
                height_cm = $7,
                status = $8,
                metadata = metadata || $9,
                updated_at = now()
            WHERE id = $1
            RETURNING id, canonical_name, normalized_name, date_of_birth,
                      nationality_code, preferred_foot, height_cm, status, created_at
            "#,
        )
        .bind(player_id)
        .bind(canonical_name)
        .bind(&normalized_name)
        .bind(draft.date_of_birth)
        .bind(
            draft
                .nationality_code
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty()),
        )
        .bind(draft.preferred_foot.as_str())
        .bind(draft.height_cm)
        .bind(draft.status.as_str())
        .bind(&draft.metadata)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| PersistenceError::InvalidState("球员不存在".to_string()))?;
        if previous_normalized_name != normalized_name {
            sqlx::query("UPDATE football.player_names SET is_primary = false WHERE player_id = $1")
                .bind(player_id)
                .execute(&mut *tx)
                .await?;
            sqlx::query(
                r#"
                INSERT INTO football.player_names (
                    id, player_id, name, normalized_name, is_primary
                ) VALUES ($1, $2, $3, $4, true)
                "#,
            )
            .bind(Uuid::new_v4())
            .bind(player_id)
            .bind(canonical_name)
            .bind(&normalized_name)
            .execute(&mut *tx)
            .await?;
        }
        sqlx::query(
            r#"
            INSERT INTO audit.events (id, event_type, entity_type, entity_id, payload)
            VALUES ($1, 'player_updated', 'player', $2, $3)
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(player_id.to_string())
        .bind(json!({"canonical_name": canonical_name, "source": "manual"}))
        .execute(&mut *tx)
        .await?;
        tx.commit().await?;
        player_record_from_row(&row)
    }

    pub async fn delete_player(&self, player_id: Uuid) -> PersistenceResult<()> {
        let check = self.check_entity_deletion("player", player_id).await?;
        if !check.can_permanently_delete {
            return Err(PersistenceError::InvalidState(check.reason));
        }
        let mut tx = self.pool.begin().await?;
        let player_name: String = sqlx::query_scalar(
            "SELECT canonical_name FROM football.players WHERE id = $1 FOR UPDATE",
        )
        .bind(player_id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| PersistenceError::InvalidState("球员不存在".to_string()))?;

        sqlx::query("DELETE FROM football.external_entity_ids WHERE entity_type = 'player' AND entity_id = $1")
            .bind(player_id)
            .execute(&mut *tx)
            .await?;
        audit_in_tx(
            &mut tx,
            "player_deleted",
            "player",
            player_id.to_string(),
            json!({"canonical_name": player_name, "reference_check": "passed"}),
        )
        .await?;
        sqlx::query("DELETE FROM football.players WHERE id = $1")
            .bind(player_id)
            .execute(&mut *tx)
            .await?;
        tx.commit().await?;
        Ok(())
    }

    pub async fn list_players(&self, query: &PlayerListQuery) -> PersistenceResult<PlayerListPage> {
        let limit = query.limit.clamp(1, 200);
        if query.cursor_name.is_some() != query.cursor_id.is_some() {
            return Err(PersistenceError::InvalidState(
                "球员分页游标必须同时包含名称和 ID".to_string(),
            ));
        }
        let mut builder: QueryBuilder<Postgres> = QueryBuilder::new(
            r#"
            SELECT
                player.id,
                player.canonical_name,
                localized_name.name AS localized_name,
                alternate_name.name AS alternate_name,
                player.normalized_name,
                player.date_of_birth,
                player.nationality_code,
                player.preferred_foot,
                player.status,
                current_team.team_id AS current_team_id,
                current_team.team_name AS current_team_name,
                primary_position.position_code AS primary_position_code,
                primary_position.default_role_code AS primary_role_code,
                COALESCE(position_roles.position_role_map, '{}'::jsonb) AS position_role_map,
                current_availability.status AS availability_status,
                current_availability.reason AS availability_reason,
                current_availability.confidence AS availability_confidence,
                current_availability.valid_to AS availability_valid_to,
                current_availability.competition_name AS availability_competition_name,
                ability.average_value AS ability_average,
                ability.average_confidence AS ability_confidence,
                COALESCE(ability.dimension_count, 0) AS ability_dimension_count
            FROM football.players player
            LEFT JOIN LATERAL (
                SELECT alias.name
                FROM football.player_names alias
                WHERE alias.player_id = player.id
                  AND (
                    lower(COALESCE(alias.language_code, '')) IN ('zh-cn', 'zh-hans', 'zh')
                    OR alias.name ~ '[一-龥]'
                  )
                ORDER BY
                  CASE lower(COALESCE(alias.language_code, ''))
                    WHEN 'zh-cn' THEN 0 WHEN 'zh-hans' THEN 1 WHEN 'zh' THEN 2 ELSE 3
                  END,
                  alias.is_primary DESC,
                  alias.valid_from DESC NULLS LAST,
                  alias.id DESC
                LIMIT 1
            ) localized_name ON true
            LEFT JOIN LATERAL (
                SELECT alias.name
                FROM football.player_names alias
                WHERE alias.player_id = player.id
                  AND alias.name <> player.canonical_name
                  AND NOT (
                    lower(COALESCE(alias.language_code, '')) IN ('zh-cn', 'zh-hans', 'zh')
                    OR alias.name ~ '[一-龥]'
                  )
                ORDER BY
                  CASE lower(COALESCE(alias.language_code, ''))
                    WHEN 'en' THEN 0 WHEN 'pt' THEN 1 WHEN 'es' THEN 2 ELSE 3
                  END,
                  alias.is_primary DESC,
                  alias.valid_from DESC NULLS LAST,
                  alias.id DESC
                LIMIT 1
            ) alternate_name ON true
            LEFT JOIN LATERAL (
                SELECT period.team_id, team.canonical_name AS team_name
                FROM football.player_team_periods period
                JOIN football.teams team ON team.id = period.team_id
                WHERE period.player_id = player.id
                  AND period.valid_from <= current_date
                  AND (period.valid_to IS NULL OR period.valid_to >= current_date)
                  AND period.registration_status IN ('registered', 'loan', 'trial')
                ORDER BY period.valid_from DESC, period.id DESC
                LIMIT 1
            ) current_team ON true
            LEFT JOIN LATERAL (
                SELECT position.position_code, position.default_role_code
                FROM football.player_positions position
                WHERE position.player_id = player.id
                  AND (position.valid_from IS NULL OR position.valid_from <= current_date)
                  AND (position.valid_to IS NULL OR position.valid_to >= current_date)
                ORDER BY position.is_primary DESC, position.proficiency DESC, position.position_code
                LIMIT 1
            ) primary_position ON true
            LEFT JOIN LATERAL (
                SELECT jsonb_object_agg(position.position_code, position.default_role_code)
                       FILTER (WHERE position.default_role_code IS NOT NULL) AS position_role_map
                FROM football.player_positions position
                WHERE position.player_id = player.id
                  AND (position.valid_from IS NULL OR position.valid_from <= current_date)
                  AND (position.valid_to IS NULL OR position.valid_to >= current_date)
            ) position_roles ON true
            LEFT JOIN LATERAL (
                SELECT availability.status, availability.reason, availability.confidence,
                       availability.valid_to, competition.name AS competition_name
                FROM football.player_availability availability
                LEFT JOIN football.competitions competition ON competition.id = availability.competition_id
                WHERE availability.player_id = player.id
                  AND availability.valid_from <= now()
                  AND (availability.valid_to IS NULL OR availability.valid_to >= now())
                ORDER BY availability.valid_from DESC, availability.created_at DESC
                LIMIT 1
            ) current_availability ON true
            LEFT JOIN feature.player_ability_profiles ability
              ON ability.player_id = player.id
             AND (ability.next_expiry_at IS NULL OR ability.next_expiry_at >= now())
            WHERE 1 = 1
            "#,
        );

        if let Some(search) = NameSearch::parse(query.search.as_deref()) {
            push_name_search(
                &mut builder,
                &search,
                NameSearchColumns {
                    primary_normalized: "player.normalized_name",
                    primary_display: "player.canonical_name",
                    alias_table: "football.player_names",
                    alias_owner: "alias.player_id",
                    owner_id: "player.id",
                    alias_normalized: "alias.normalized_name",
                    alias_display: "alias.name",
                },
            );
        }
        if let Some(team_id) = query.team_id {
            builder.push(
                " AND EXISTS (SELECT 1 FROM football.player_team_periods filter_period WHERE filter_period.player_id = player.id AND filter_period.team_id = ",
            );
            builder.push_bind(team_id);
            builder.push(" AND filter_period.valid_from <= current_date AND (filter_period.valid_to IS NULL OR filter_period.valid_to >= current_date) AND filter_period.registration_status IN ('registered', 'loan', 'trial'))");
        }
        if let Some(position_code) = query
            .position_code
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
        {
            builder.push(" AND EXISTS (SELECT 1 FROM football.player_positions filter_position WHERE filter_position.player_id = player.id AND filter_position.position_code = ");
            builder.push_bind(position_code.to_uppercase());
            builder.push(" AND (filter_position.valid_from IS NULL OR filter_position.valid_from <= current_date) AND (filter_position.valid_to IS NULL OR filter_position.valid_to >= current_date))");
        }
        if let Some(status) = query.availability_status {
            builder.push(" AND EXISTS (SELECT 1 FROM football.player_availability filter_availability WHERE filter_availability.player_id = player.id AND filter_availability.status = ");
            builder.push_bind(status.as_str());
            builder.push(" AND filter_availability.valid_from <= now() AND (filter_availability.valid_to IS NULL OR filter_availability.valid_to >= now()))");
        }
        if let Some(status) = query.player_status {
            builder.push(" AND player.status = ");
            builder.push_bind(status.as_str());
        }
        if let (Some(cursor_name), Some(cursor_id)) = (&query.cursor_name, query.cursor_id) {
            builder.push(" AND (player.normalized_name, player.id) > (");
            builder.push_bind(cursor_name);
            builder.push(", ");
            builder.push_bind(cursor_id);
            builder.push(")");
        }
        builder.push(" ORDER BY player.normalized_name, player.id LIMIT ");
        builder.push_bind(i64::from(limit) + 1);

        let rows = builder.build().fetch_all(&self.pool).await?;
        let has_more = rows.len() > limit as usize;
        let mut items: Vec<PlayerListItem> = rows
            .iter()
            .take(limit as usize)
            .map(player_list_item_from_row)
            .collect::<PersistenceResult<_>>()?;
        let (next_cursor_name, next_cursor_id) = if has_more {
            items
                .last()
                .map(|item| (Some(item.normalized_name.clone()), Some(item.id)))
                .unwrap_or((None, None))
        } else {
            (None, None)
        };
        items.shrink_to_fit();
        Ok(PlayerListPage {
            items,
            next_cursor_name,
            next_cursor_id,
            has_more,
        })
    }

    pub async fn read_player(&self, player_id: Uuid) -> PersistenceResult<PlayerDetail> {
        let row = sqlx::query(
            r#"
            SELECT
                id, canonical_name, normalized_name, date_of_birth,
                nationality_code, preferred_foot, height_cm, status, created_at
            FROM football.players
            WHERE id = $1
            "#,
        )
        .bind(player_id)
        .fetch_one(&self.pool)
        .await?;
        let player = player_record_from_row(&row)?;

        let name_rows = sqlx::query(
            r#"
            SELECT id, player_id, name, normalized_name, language_code,
                   is_primary, valid_from, valid_to
            FROM football.player_names
            WHERE player_id = $1
            ORDER BY is_primary DESC, valid_from DESC NULLS LAST, name
            "#,
        )
        .bind(player_id)
        .fetch_all(&self.pool)
        .await?;
        let names = name_rows
            .iter()
            .map(player_name_from_row)
            .collect::<PersistenceResult<Vec<_>>>()?;

        let position_rows = sqlx::query(
            r#"
            SELECT
                assignment.id, assignment.player_id, assignment.position_code,
                position.name AS position_name, position.position_group,
                assignment.proficiency, assignment.default_role_code, assignment.is_primary,
                assignment.valid_from, assignment.valid_to
            FROM football.player_positions assignment
            JOIN football.positions position ON position.code = assignment.position_code
            WHERE assignment.player_id = $1
            ORDER BY assignment.is_primary DESC, assignment.proficiency DESC,
                     assignment.valid_from DESC NULLS LAST
            "#,
        )
        .bind(player_id)
        .fetch_all(&self.pool)
        .await?;
        let positions = position_rows
            .iter()
            .map(player_position_from_row)
            .collect::<PersistenceResult<Vec<_>>>()?;

        let period_rows = sqlx::query(
            r#"
            SELECT
                period.id, period.player_id, period.team_id,
                team.canonical_name AS team_name,
                period.season_id, season.name AS season_name,
                period.squad_number, period.valid_from, period.valid_to,
                period.registration_status
            FROM football.player_team_periods period
            JOIN football.teams team ON team.id = period.team_id
            LEFT JOIN football.seasons season ON season.id = period.season_id
            WHERE period.player_id = $1
            ORDER BY period.valid_from DESC, period.id DESC
            "#,
        )
        .bind(player_id)
        .fetch_all(&self.pool)
        .await?;
        let team_periods = period_rows
            .iter()
            .map(player_team_period_from_row)
            .collect::<PersistenceResult<Vec<_>>>()?;

        let availability_rows = sqlx::query(
            r#"
            SELECT
                availability.id, availability.player_id, availability.team_id,
                team.canonical_name AS team_name, availability.competition_id,
                availability.status, availability.reason, availability.confidence,
                availability.valid_from, availability.valid_to, availability.created_at
            FROM football.player_availability availability
            LEFT JOIN football.teams team ON team.id = availability.team_id
            WHERE availability.player_id = $1
            ORDER BY availability.valid_from DESC, availability.created_at DESC
            LIMIT 100
            "#,
        )
        .bind(player_id)
        .fetch_all(&self.pool)
        .await?;
        let availability = availability_rows
            .iter()
            .map(player_availability_from_row)
            .collect::<PersistenceResult<Vec<_>>>()?;

        let ability_profile = sqlx::query(
            r#"
            SELECT player_id, abilities, average_value, average_confidence,
                   dimension_count, latest_observed_at, next_expiry_at, updated_at
            FROM feature.player_ability_profiles
            WHERE player_id = $1
              AND (next_expiry_at IS NULL OR next_expiry_at >= now())
            "#,
        )
        .bind(player_id)
        .fetch_optional(&self.pool)
        .await?
        .as_ref()
        .map(player_ability_profile_from_row)
        .transpose()?;

        let observation_rows = sqlx::query(
            r#"
            SELECT
                observation.id, observation.player_id, observation.dimension_code,
                dimension.name AS dimension_name, observation.context_type,
                observation.context_id, observation.value, observation.confidence,
                observation.sample_size, observation.observed_at,
                observation.effective_from, observation.effective_to,
                observation.calculation_version
            FROM feature.player_ability_observations observation
            JOIN feature.player_ability_dimensions dimension
              ON dimension.code = observation.dimension_code
            WHERE observation.player_id = $1
            ORDER BY observation.observed_at DESC, observation.id DESC
            LIMIT 250
            "#,
        )
        .bind(player_id)
        .fetch_all(&self.pool)
        .await?;
        let ability_observations = observation_rows
            .iter()
            .map(player_ability_observation_from_row)
            .collect::<PersistenceResult<Vec<_>>>()?;

        let dynamic_tags = self.list_player_dynamic_tags(player_id, Utc::now()).await?;

        let external_rows = sqlx::query(
            r#"
            SELECT external.id, external.provider_id, provider.name AS provider_name,
                   external.entity_type, external.entity_id, external.external_id,
                   external.metadata
            FROM football.external_entity_ids external
            JOIN catalog.data_providers provider ON provider.id = external.provider_id
            WHERE external.entity_type = 'player' AND external.entity_id = $1
            ORDER BY provider.name, external.external_id
            "#,
        )
        .bind(player_id)
        .fetch_all(&self.pool)
        .await?;
        let external_ids = external_rows
            .iter()
            .map(external_entity_id_from_row)
            .collect::<PersistenceResult<Vec<_>>>()?;

        Ok(PlayerDetail {
            player,
            names,
            positions,
            team_periods,
            availability,
            ability_profile,
            ability_observations,
            dynamic_tags,
            external_ids,
        })
    }

    pub async fn add_player_name(
        &self,
        draft: &PlayerNameDraft,
    ) -> PersistenceResult<PlayerNameRecord> {
        let name = draft.name.trim();
        if name.is_empty() {
            return Err(PersistenceError::InvalidState(
                "球员名称不能为空".to_string(),
            ));
        }
        if matches!((&draft.valid_from, &draft.valid_to), (Some(start), Some(end)) if end < start) {
            return Err(PersistenceError::InvalidState(
                "球员名称结束日期不能早于开始日期".to_string(),
            ));
        }
        let mut tx = self.pool.begin().await?;
        if draft.is_primary {
            sqlx::query("UPDATE football.player_names SET is_primary = false WHERE player_id = $1")
                .bind(draft.player_id)
                .execute(&mut *tx)
                .await?;
            sqlx::query(
                r#"
                UPDATE football.players
                SET canonical_name = $2, normalized_name = $3, updated_at = now()
                WHERE id = $1
                "#,
            )
            .bind(draft.player_id)
            .bind(name)
            .bind(normalize_name(name))
            .execute(&mut *tx)
            .await?;
        }
        let row = sqlx::query(
            r#"
            INSERT INTO football.player_names (
                id, player_id, name, normalized_name, language_code,
                is_primary, valid_from, valid_to
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING id, player_id, name, normalized_name, language_code,
                      is_primary, valid_from, valid_to
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(draft.player_id)
        .bind(name)
        .bind(normalize_name(name))
        .bind(
            draft
                .language_code
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty()),
        )
        .bind(draft.is_primary)
        .bind(draft.valid_from)
        .bind(draft.valid_to)
        .fetch_one(&mut *tx)
        .await?;
        tx.commit().await?;
        player_name_from_row(&row)
    }

    pub async fn assign_player_position(
        &self,
        draft: &PlayerPositionDraft,
    ) -> PersistenceResult<PlayerPositionRecord> {
        if !(0.0..=1.0).contains(&draft.proficiency) {
            return Err(PersistenceError::InvalidState(
                "位置熟练度必须位于 0–1".to_string(),
            ));
        }
        if matches!((&draft.valid_from, &draft.valid_to), (Some(start), Some(end)) if end < start) {
            return Err(PersistenceError::InvalidState(
                "球员位置结束日期不能早于开始日期".to_string(),
            ));
        }
        if draft
            .default_role_code
            .as_deref()
            .is_some_and(|value| value.trim().chars().count() > 80)
        {
            return Err(PersistenceError::InvalidState(
                "默认战术角色不能超过 80 个字符".to_string(),
            ));
        }
        let position_code = draft.position_code.trim().to_uppercase();
        let mut tx = self.pool.begin().await?;
        if draft.is_primary {
            sqlx::query(
                "UPDATE football.player_positions SET is_primary = false WHERE player_id = $1",
            )
            .bind(draft.player_id)
            .execute(&mut *tx)
            .await?;
        }
        let row = sqlx::query(
            r#"
            WITH inserted AS (
                INSERT INTO football.player_positions (
                    id, player_id, position_code, proficiency, default_role_code, is_primary,
                    valid_from, valid_to, source_document_id
                ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
                RETURNING *
            )
            SELECT
                inserted.id, inserted.player_id, inserted.position_code,
                position.name AS position_name, position.position_group,
                inserted.proficiency, inserted.default_role_code, inserted.is_primary,
                inserted.valid_from, inserted.valid_to
            FROM inserted
            JOIN football.positions position ON position.code = inserted.position_code
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(draft.player_id)
        .bind(&position_code)
        .bind(draft.proficiency)
        .bind(
            draft
                .default_role_code
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty()),
        )
        .bind(draft.is_primary)
        .bind(draft.valid_from)
        .bind(draft.valid_to)
        .bind(draft.source_document_id)
        .fetch_one(&mut *tx)
        .await?;
        tx.commit().await?;
        player_position_from_row(&row)
    }

    pub async fn add_player_team_period(
        &self,
        draft: &PlayerTeamPeriodDraft,
    ) -> PersistenceResult<PlayerTeamPeriodRecord> {
        if draft
            .valid_to
            .as_ref()
            .is_some_and(|valid_to| valid_to < &draft.valid_from)
        {
            return Err(PersistenceError::InvalidState(
                "球队效力结束日期不能早于开始日期".to_string(),
            ));
        }
        if draft
            .squad_number
            .is_some_and(|number| !(0..=99).contains(&number))
        {
            return Err(PersistenceError::InvalidState(
                "球衣号码必须位于 0–99".to_string(),
            ));
        }
        let row = sqlx::query(
            r#"
            WITH inserted AS (
                INSERT INTO football.player_team_periods (
                    id, player_id, team_id, season_id, squad_number,
                    valid_from, valid_to, registration_status, source_document_id
                ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
                RETURNING *
            )
            SELECT
                inserted.id, inserted.player_id, inserted.team_id,
                team.canonical_name AS team_name,
                inserted.season_id, season.name AS season_name,
                inserted.squad_number, inserted.valid_from, inserted.valid_to,
                inserted.registration_status
            FROM inserted
            JOIN football.teams team ON team.id = inserted.team_id
            LEFT JOIN football.seasons season ON season.id = inserted.season_id
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(draft.player_id)
        .bind(draft.team_id)
        .bind(draft.season_id)
        .bind(draft.squad_number)
        .bind(draft.valid_from)
        .bind(draft.valid_to)
        .bind(draft.registration_status.trim())
        .bind(draft.source_document_id)
        .fetch_one(&self.pool)
        .await?;
        player_team_period_from_row(&row)
    }

    pub async fn add_player_availability(
        &self,
        draft: &PlayerAvailabilityDraft,
    ) -> PersistenceResult<PlayerAvailabilityRecord> {
        if !(0.0..=1.0).contains(&draft.confidence) {
            return Err(PersistenceError::InvalidState(
                "可用性可信度必须位于 0–1".to_string(),
            ));
        }
        if draft
            .valid_to
            .as_ref()
            .is_some_and(|valid_to| valid_to < &draft.valid_from)
        {
            return Err(PersistenceError::InvalidState(
                "可用性结束时间不能早于开始时间".to_string(),
            ));
        }
        let row = sqlx::query(
            r#"
            WITH inserted AS (
                INSERT INTO football.player_availability (
                    id, player_id, team_id, competition_id, status, reason,
                    confidence, valid_from, valid_to, source_document_id, metadata
                ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
                RETURNING *
            )
            SELECT
                inserted.id, inserted.player_id, inserted.team_id,
                team.canonical_name AS team_name, inserted.competition_id,
                inserted.status, inserted.reason, inserted.confidence,
                inserted.valid_from, inserted.valid_to, inserted.created_at
            FROM inserted
            LEFT JOIN football.teams team ON team.id = inserted.team_id
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(draft.player_id)
        .bind(draft.team_id)
        .bind(draft.competition_id)
        .bind(draft.status.as_str())
        .bind(
            draft
                .reason
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty()),
        )
        .bind(draft.confidence)
        .bind(draft.valid_from)
        .bind(draft.valid_to)
        .bind(draft.source_document_id)
        .bind(&draft.metadata)
        .fetch_one(&self.pool)
        .await?;
        player_availability_from_row(&row)
    }

    pub async fn add_player_ability_observation(
        &self,
        draft: &PlayerAbilityObservationDraft,
    ) -> PersistenceResult<PlayerAbilityObservationRecord> {
        if !(0.0..=1.0).contains(&draft.confidence) || draft.sample_size < 0 {
            return Err(PersistenceError::InvalidState(
                "能力观察可信度或样本量无效".to_string(),
            ));
        }
        if draft
            .effective_to
            .as_ref()
            .is_some_and(|value| value < &draft.effective_from)
        {
            return Err(PersistenceError::InvalidState(
                "能力观察失效时间不能早于生效时间".to_string(),
            ));
        }
        let row = sqlx::query(
            r#"
            WITH dimension AS (
                SELECT code, name, minimum_value, maximum_value
                FROM feature.player_ability_dimensions
                WHERE code = $2
            ), inserted AS (
                INSERT INTO feature.player_ability_observations (
                    id, player_id, dimension_code, context_type, context_id,
                    value, confidence, sample_size, observed_at,
                    effective_from, effective_to, calculation_version,
                    source_document_id, metadata
                )
                SELECT
                    $1, $3, dimension.code, $4, $5, $6, $7, $8, $9,
                    $10, $11, $12, $13, $14
                FROM dimension
                WHERE $6 BETWEEN dimension.minimum_value AND dimension.maximum_value
                RETURNING *
            )
            SELECT
                inserted.id, inserted.player_id, inserted.dimension_code,
                dimension.name AS dimension_name, inserted.context_type,
                inserted.context_id, inserted.value, inserted.confidence,
                inserted.sample_size, inserted.observed_at,
                inserted.effective_from, inserted.effective_to,
                inserted.calculation_version
            FROM inserted
            JOIN dimension ON dimension.code = inserted.dimension_code
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(draft.dimension_code.trim())
        .bind(draft.player_id)
        .bind(draft.context_type.trim())
        .bind(draft.context_id)
        .bind(draft.value)
        .bind(draft.confidence)
        .bind(draft.sample_size)
        .bind(draft.observed_at)
        .bind(draft.effective_from)
        .bind(draft.effective_to)
        .bind(draft.calculation_version.trim())
        .bind(draft.source_document_id)
        .bind(&draft.metadata)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| {
            PersistenceError::InvalidState("能力维度不存在，或能力值超出该维度允许范围".to_string())
        })?;
        player_ability_observation_from_row(&row)
    }

    pub async fn add_external_entity_id(
        &self,
        draft: &ExternalEntityIdDraft,
    ) -> PersistenceResult<ExternalEntityIdRecord> {
        if !matches!(
            draft.entity_type.as_str(),
            "competition" | "season" | "team" | "player" | "coach" | "match"
        ) {
            return Err(PersistenceError::InvalidState(
                "外部 ID 实体类型无效".to_string(),
            ));
        }
        if draft.external_id.trim().is_empty() {
            return Err(PersistenceError::InvalidState(
                "外部 ID 不能为空".to_string(),
            ));
        }
        let row = sqlx::query(
            r#"
            WITH inserted AS (
                INSERT INTO football.external_entity_ids (
                    id, provider_id, entity_type, entity_id, external_id, metadata
                ) VALUES ($1, $2, $3, $4, $5, $6)
                ON CONFLICT (provider_id, entity_type, external_id) DO UPDATE SET
                    entity_id = EXCLUDED.entity_id,
                    metadata = football.external_entity_ids.metadata || EXCLUDED.metadata
                RETURNING *
            )
            SELECT inserted.id, inserted.provider_id,
                   provider.name AS provider_name, inserted.entity_type,
                   inserted.entity_id, inserted.external_id, inserted.metadata
            FROM inserted
            JOIN catalog.data_providers provider ON provider.id = inserted.provider_id
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(draft.provider_id)
        .bind(draft.entity_type.trim())
        .bind(draft.entity_id)
        .bind(draft.external_id.trim())
        .bind(&draft.metadata)
        .fetch_one(&self.pool)
        .await?;
        external_entity_id_from_row(&row)
    }

    pub async fn create_match(&self, draft: &MatchDraft) -> PersistenceResult<MatchRecord> {
        let resolved = resolve_match_scope_draft(&self.pool, draft).await?;
        let draft = &resolved;
        if draft.home_team_id == draft.away_team_id {
            return Err(PersistenceError::InvalidState(
                "主队和客队不能相同".to_string(),
            ));
        }
        validate_match_scope(&self.pool, draft).await?;
        let external_key = if draft.external_key.trim().is_empty() {
            let kickoff = draft.kickoff_time.format("%Y%m%dT%H%MZ");
            let home = draft.home_team_id.simple().to_string();
            let away = draft.away_team_id.simple().to_string();
            format!("MATCH-{kickoff}-{}-{}", &home[..8], &away[..8])
        } else {
            draft.external_key.trim().to_string()
        };
        let generated_id = Uuid::new_v4();
        let row = sqlx::query(
            r#"
            WITH upserted AS (
                INSERT INTO football.matches (
                    id, external_key, competition_id, season_id, stage_id, round_id,
                    home_team_id, away_team_id, kickoff_time, status, venue, metadata
                ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
                ON CONFLICT (external_key) DO UPDATE SET
                    competition_id = EXCLUDED.competition_id,
                    season_id = EXCLUDED.season_id,
                    stage_id = EXCLUDED.stage_id,
                    round_id = EXCLUDED.round_id,
                    home_team_id = EXCLUDED.home_team_id,
                    away_team_id = EXCLUDED.away_team_id,
                    kickoff_time = EXCLUDED.kickoff_time,
                    status = EXCLUDED.status,
                    venue = EXCLUDED.venue,
                    metadata = football.matches.metadata || EXCLUDED.metadata,
                    updated_at = now()
                RETURNING *
            )
            SELECT
                upserted.id, upserted.external_key,
                upserted.competition_id, competition.name AS competition_name,
                upserted.season_id, upserted.stage_id, upserted.round_id,
                upserted.home_team_id, home.canonical_name AS home_team_name,
                upserted.away_team_id, away.canonical_name AS away_team_name,
                upserted.kickoff_time, upserted.status, upserted.venue
            FROM upserted
            LEFT JOIN football.competitions competition ON competition.id = upserted.competition_id
            JOIN football.teams home ON home.id = upserted.home_team_id
            JOIN football.teams away ON away.id = upserted.away_team_id
            "#,
        )
        .bind(generated_id)
        .bind(&external_key)
        .bind(draft.competition_id)
        .bind(draft.season_id)
        .bind(draft.stage_id)
        .bind(draft.round_id)
        .bind(draft.home_team_id)
        .bind(draft.away_team_id)
        .bind(draft.kickoff_time)
        .bind(draft.status.as_str())
        .bind(
            draft
                .venue
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty()),
        )
        .bind(&draft.metadata)
        .fetch_one(&self.pool)
        .await?;
        match_record_from_row(&row)
    }

    pub async fn delete_match(&self, match_id: Uuid) -> PersistenceResult<()> {
        let mut tx = self.pool.begin().await?;
        let match_key: String = sqlx::query_scalar(
            "SELECT external_key FROM football.matches WHERE id = $1 FOR UPDATE",
        )
        .bind(match_id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| PersistenceError::InvalidState("比赛不存在".to_string()))?;
        let protected_count = sqlx::query_scalar::<_, i64>(
            r#"
            SELECT
                (SELECT count(*) FROM research.runs WHERE match_id = $1)
              + (SELECT count(*) FROM platform.p4_freeze_tasks WHERE match_id = $1)
              + (SELECT count(*) FROM review.postmatch_settlements WHERE match_id = $1)
            "#,
        )
        .bind(match_id)
        .fetch_one(&mut *tx)
        .await?;
        if protected_count > 0 {
            return Err(PersistenceError::InvalidState(
                "该比赛已进入P4研究、冻结或正式赛后结算，必须保留不可变审计血缘，不能永久删除"
                    .to_string(),
            ));
        }

        sqlx::query("DELETE FROM football.external_entity_ids WHERE entity_type = 'match' AND entity_id = $1")
            .bind(match_id).execute(&mut *tx).await?;
        sqlx::query("UPDATE model.runs SET match_id = NULL WHERE match_id = $1")
            .bind(match_id)
            .execute(&mut *tx)
            .await?;
        sqlx::query("UPDATE feature.snapshots SET match_id = NULL WHERE match_id = $1")
            .bind(match_id)
            .execute(&mut *tx)
            .await?;
        sqlx::query("UPDATE ai_workspace.sessions SET match_id = NULL WHERE match_id = $1")
            .bind(match_id)
            .execute(&mut *tx)
            .await?;
        sqlx::query(
            "DELETE FROM review.ability_update_candidates WHERE match_review_id IN (SELECT id FROM review.match_reviews WHERE match_id = $1)",
        )
        .bind(match_id)
        .execute(&mut *tx)
        .await?;
        sqlx::query("DELETE FROM review.match_reviews WHERE match_id = $1")
            .bind(match_id)
            .execute(&mut *tx)
            .await?;
        audit_in_tx(
            &mut tx,
            "match_deleted",
            "match",
            match_id.to_string(),
            json!({"external_key": match_key}),
        )
        .await?;
        sqlx::query("DELETE FROM football.matches WHERE id = $1")
            .bind(match_id)
            .execute(&mut *tx)
            .await?;
        tx.commit().await?;
        Ok(())
    }

    pub async fn list_upcoming_matches(&self, limit: u32) -> PersistenceResult<Vec<MatchRecord>> {
        let rows = sqlx::query(
            r#"
            SELECT
                fixture.id, fixture.external_key,
                fixture.competition_id, competition.name AS competition_name,
                fixture.season_id, fixture.stage_id, fixture.round_id,
                fixture.home_team_id, home.canonical_name AS home_team_name,
                fixture.away_team_id, away.canonical_name AS away_team_name,
                fixture.kickoff_time, fixture.status, fixture.venue
            FROM football.matches fixture
            LEFT JOIN football.competitions competition ON competition.id = fixture.competition_id
            JOIN football.teams home ON home.id = fixture.home_team_id
            JOIN football.teams away ON away.id = fixture.away_team_id
            WHERE fixture.status IN ('scheduled', 'live')
            ORDER BY fixture.kickoff_time, fixture.id
            LIMIT $1
            "#,
        )
        .bind(i64::from(limit.clamp(1, 250)))
        .fetch_all(&self.pool)
        .await?;
        rows.iter().map(match_record_from_row).collect()
    }

    pub async fn list_managed_matches(&self, limit: u32) -> PersistenceResult<Vec<MatchRecord>> {
        let rows = sqlx::query(
            r#"
            SELECT
                fixture.id, fixture.external_key,
                fixture.competition_id, competition.name AS competition_name,
                fixture.season_id, fixture.stage_id, fixture.round_id,
                fixture.home_team_id, home.canonical_name AS home_team_name,
                fixture.away_team_id, away.canonical_name AS away_team_name,
                fixture.kickoff_time, fixture.status, fixture.venue
            FROM football.matches fixture
            LEFT JOIN football.competitions competition ON competition.id = fixture.competition_id
            JOIN football.teams home ON home.id = fixture.home_team_id
            JOIN football.teams away ON away.id = fixture.away_team_id
            ORDER BY fixture.kickoff_time DESC, fixture.id DESC
            LIMIT $1
            "#,
        )
        .bind(i64::from(limit.clamp(1, 500)))
        .fetch_all(&self.pool)
        .await?;
        rows.iter().map(match_record_from_row).collect()
    }

    pub async fn create_lineup(&self, draft: &LineupDraft) -> PersistenceResult<LineupRecord> {
        let validated = validate_lineup_draft(draft)?;
        let mut tx = self.pool.begin().await?;
        let lineup_id = insert_lineup_in_tx(&mut tx, draft, &validated).await?;
        tx.commit().await?;
        self.read_lineup(lineup_id).await
    }

    pub async fn create_lineup_pair(
        &self,
        draft: &LineupPairDraft,
    ) -> PersistenceResult<LineupPairRecord> {
        if draft.home.match_id != draft.away.match_id {
            return Err(PersistenceError::InvalidState(
                "双方阵容必须属于同一场比赛".to_string(),
            ));
        }
        if draft.home.team_id == draft.away.team_id {
            return Err(PersistenceError::InvalidState(
                "双方阵容不能使用同一支球队".to_string(),
            ));
        }
        if draft.home.snapshot_type != draft.away.snapshot_type {
            return Err(PersistenceError::InvalidState(
                "双方阵容必须使用同一数据窗口".to_string(),
            ));
        }
        if draft.home.lineup_type != draft.away.lineup_type {
            return Err(PersistenceError::InvalidState(
                "双方阵容必须使用同一阵容类型".to_string(),
            ));
        }
        let home_validated = validate_lineup_draft(&draft.home)?;
        let away_validated = validate_lineup_draft(&draft.away)?;
        let mut tx = self.pool.begin().await?;
        let sides = sqlx::query(
            "SELECT home_team_id, away_team_id FROM football.matches WHERE id=$1 FOR UPDATE",
        )
        .bind(draft.home.match_id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| PersistenceError::InvalidState("比赛不存在".to_string()))?;
        let home_team_id: Uuid = sides.try_get("home_team_id")?;
        let away_team_id: Uuid = sides.try_get("away_team_id")?;
        if draft.home.team_id != home_team_id || draft.away.team_id != away_team_id {
            return Err(PersistenceError::InvalidState(
                "双方阵容必须分别对应比赛主队和客队".to_string(),
            ));
        }
        let home_id = insert_lineup_in_tx(&mut tx, &draft.home, &home_validated).await?;
        let away_id = insert_lineup_in_tx(&mut tx, &draft.away, &away_validated).await?;
        audit_in_tx(
            &mut tx,
            "lineup_pair_created",
            "match",
            draft.home.match_id.to_string(),
            json!({
                "home_lineup_id": home_id,
                "away_lineup_id": away_id,
                "snapshot_type": home_validated.snapshot_type,
                "lineup_type": draft.home.lineup_type.as_str(),
            }),
        )
        .await?;
        tx.commit().await?;
        Ok(LineupPairRecord {
            home: self.read_lineup(home_id).await?,
            away: self.read_lineup(away_id).await?,
        })
    }

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

    pub async fn remove_lineup_history(
        &self,
        lineup_id: Uuid,
        reason: Option<&str>,
    ) -> PersistenceResult<LineupHistoryRemovalResult> {
        let reason = reason
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .unwrap_or("用户从阵容历史中删除");
        let mut tx = self.pool.begin().await?;
        let row = sqlx::query(
            r#"
            SELECT id, match_id, team_id, snapshot_type, lineup_type, status,
                   history_hidden_at
            FROM football.lineups
            WHERE id = $1
            FOR UPDATE
            "#,
        )
        .bind(lineup_id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| PersistenceError::InvalidState("阵容版本不存在".to_string()))?;

        if row
            .try_get::<Option<chrono::DateTime<chrono::Utc>>, _>("history_hidden_at")?
            .is_some()
        {
            return Err(PersistenceError::InvalidState(
                "阵容版本已经从历史列表隐藏".to_string(),
            ));
        }

        let match_id: Uuid = row.try_get("match_id")?;
        let team_id: Uuid = row.try_get("team_id")?;
        let snapshot_type: String = row.try_get("snapshot_type")?;
        let lineup_type: String = row.try_get("lineup_type")?;
        let status: String = row.try_get("status")?;

        let referenced: bool = sqlx::query_scalar(
            r#"
            SELECT
                EXISTS (SELECT 1 FROM football.lineups WHERE supersedes_lineup_id = $1)
                OR EXISTS (SELECT 1 FROM feature.match_player_contributions WHERE lineup_id = $1)
                OR EXISTS (SELECT 1 FROM feature.snapshots WHERE input_payload::text LIKE '%' || $1::text || '%')
                OR EXISTS (SELECT 1 FROM model.runs WHERE input_payload::text LIKE '%' || $1::text || '%')
            "#,
        )
        .bind(lineup_id)
        .fetch_one(&mut *tx)
        .await?;

        let removal_mode = if referenced {
            sqlx::query(
                r#"
                UPDATE football.lineups
                SET history_hidden_at = now(),
                    history_hidden_reason = $2,
                    status = CASE WHEN status = 'active' THEN 'withdrawn' ELSE status END,
                    updated_at = now()
                WHERE id = $1
                "#,
            )
            .bind(lineup_id)
            .bind(reason)
            .execute(&mut *tx)
            .await?;
            "archived"
        } else {
            sqlx::query("DELETE FROM football.lineups WHERE id = $1")
                .bind(lineup_id)
                .execute(&mut *tx)
                .await?;
            "deleted"
        };

        let restored_lineup_id = if status == "active" {
            let candidate = sqlx::query_scalar::<_, Uuid>(
                r#"
                SELECT id
                FROM football.lineups
                WHERE match_id = $1
                  AND team_id = $2
                  AND snapshot_type = $3
                  AND lineup_type = $4
                  AND status = 'superseded'
                  AND history_hidden_at IS NULL
                ORDER BY captured_at DESC, created_at DESC, id DESC
                LIMIT 1
                FOR UPDATE
                "#,
            )
            .bind(match_id)
            .bind(team_id)
            .bind(&snapshot_type)
            .bind(&lineup_type)
            .fetch_optional(&mut *tx)
            .await?;
            if let Some(candidate_id) = candidate {
                sqlx::query(
                    "UPDATE football.lineups SET status='active', updated_at=now() WHERE id=$1",
                )
                .bind(candidate_id)
                .execute(&mut *tx)
                .await?;
                Some(candidate_id)
            } else {
                None
            }
        } else {
            None
        };

        audit_in_tx(
            &mut tx,
            "lineup_history_removed",
            "lineup",
            lineup_id.to_string(),
            json!({
                "removal_mode": removal_mode,
                "reason": reason,
                "restored_lineup_id": restored_lineup_id,
                "match_id": match_id,
                "team_id": team_id,
                "snapshot_type": snapshot_type,
                "lineup_type": lineup_type,
            }),
        )
        .await?;
        tx.commit().await?;
        Ok(LineupHistoryRemovalResult {
            lineup_id,
            removal_mode: removal_mode.to_string(),
            restored_lineup_id,
        })
    }

    pub async fn player_catalog_reference_data(
        &self,
    ) -> PersistenceResult<PlayerCatalogReferenceData> {
        let teams = self.list_team_options(None, 500).await?;
        let season_team_memberships = self.list_season_team_memberships().await?;
        let formations = self.list_formations(true).await?;
        let providers = self.list_data_providers().await?;
        let positions = self.list_positions().await?;
        let ability_dimensions = self.list_ability_dimensions().await?;
        let dynamic_tag_definitions = self.list_dynamic_tag_definitions().await?;
        let upcoming_matches = self.list_upcoming_matches(100).await?;
        let managed_matches = self.list_managed_matches(300).await?;
        Ok(PlayerCatalogReferenceData {
            teams,
            season_team_memberships,
            formations,
            providers,
            positions,
            ability_dimensions,
            dynamic_tag_definitions,
            upcoming_matches,
            managed_matches,
        })
    }

    async fn list_season_team_memberships(
        &self,
    ) -> PersistenceResult<Vec<SeasonTeamMembershipOption>> {
        let rows = sqlx::query(
            r#"
            SELECT season_id, team_id, registration_status
            FROM football.team_season_memberships
            WHERE registration_status IN ('registered', 'guest')
            ORDER BY season_id, team_id
            "#,
        )
        .fetch_all(&self.pool)
        .await?;
        rows.iter()
            .map(|row| {
                Ok(SeasonTeamMembershipOption {
                    season_id: row.try_get("season_id")?,
                    team_id: row.try_get("team_id")?,
                    registration_status: row.try_get("registration_status")?,
                })
            })
            .collect()
    }

    pub async fn list_positions(&self) -> PersistenceResult<Vec<PositionReference>> {
        let rows = sqlx::query(
            r#"
            SELECT code, name, position_group, sort_order
            FROM football.positions
            ORDER BY sort_order, code
            "#,
        )
        .fetch_all(&self.pool)
        .await?;
        rows.iter().map(position_reference_from_row).collect()
    }

    pub async fn list_ability_dimensions(&self) -> PersistenceResult<Vec<AbilityDimensionRecord>> {
        let rows = sqlx::query(
            r#"
            SELECT code, name, category, minimum_value, maximum_value, description
            FROM feature.player_ability_dimensions
            ORDER BY category, code
            "#,
        )
        .fetch_all(&self.pool)
        .await?;
        rows.iter().map(ability_dimension_from_row).collect()
    }

    async fn audit(
        &self,
        event_type: &str,
        entity_type: &str,
        entity_id: String,
        payload: Value,
    ) -> PersistenceResult<()> {
        sqlx::query(
            r#"
            INSERT INTO audit.events (id, event_type, entity_type, entity_id, payload)
            VALUES ($1, $2, $3, $4, $5)
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(event_type)
        .bind(entity_type)
        .bind(entity_id)
        .bind(payload)
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}

async fn resolve_match_scope_draft(
    pool: &sqlx::PgPool,
    draft: &MatchDraft,
) -> PersistenceResult<MatchDraft> {
    let mut resolved = draft.clone();
    if let Some(round_id) = resolved.round_id {
        let row = sqlx::query(
            r#"
            SELECT round.stage_id, stage.season_id, season.competition_id
            FROM football.rounds round
            JOIN football.competition_stages stage ON stage.id = round.stage_id
            JOIN football.seasons season ON season.id = stage.season_id
            WHERE round.id = $1
            "#,
        )
        .bind(round_id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| PersistenceError::InvalidState("比赛轮次不存在".to_string()))?;
        resolved.stage_id.get_or_insert(row.try_get("stage_id")?);
        resolved.season_id.get_or_insert(row.try_get("season_id")?);
        resolved
            .competition_id
            .get_or_insert(row.try_get("competition_id")?);
    } else if let Some(stage_id) = resolved.stage_id {
        let row = sqlx::query(
            r#"
            SELECT stage.season_id, season.competition_id
            FROM football.competition_stages stage
            JOIN football.seasons season ON season.id = stage.season_id
            WHERE stage.id = $1
            "#,
        )
        .bind(stage_id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| PersistenceError::InvalidState("比赛阶段不存在".to_string()))?;
        resolved.season_id.get_or_insert(row.try_get("season_id")?);
        resolved
            .competition_id
            .get_or_insert(row.try_get("competition_id")?);
    } else if let Some(season_id) = resolved.season_id {
        let competition_id = sqlx::query_scalar::<_, Uuid>(
            "SELECT competition_id FROM football.seasons WHERE id = $1",
        )
        .bind(season_id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| PersistenceError::InvalidState("比赛赛季不存在".to_string()))?;
        resolved.competition_id.get_or_insert(competition_id);
    } else if let Some(competition_id) = resolved.competition_id {
        let competition = sqlx::query(
            r#"
            SELECT timezone,
                   ($2::timestamptz AT TIME ZONE timezone)::date AS local_kickoff_date,
                   COALESCE(NULLIF(metadata->>'season_pattern', ''), 'calendar') AS season_pattern
            FROM football.competitions
            WHERE id = $1 AND is_active
            "#,
        )
        .bind(competition_id)
        .bind(resolved.kickoff_time)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| PersistenceError::InvalidState("比赛赛事不存在或已停用".to_string()))?;
        let timezone: String = competition.try_get("timezone")?;
        let kickoff_date: NaiveDate = competition.try_get("local_kickoff_date")?;
        let season_pattern: String = competition.try_get("season_pattern")?;
        resolved.season_id = sqlx::query_scalar::<_, Uuid>(
            r#"
            SELECT id
            FROM football.seasons
            WHERE competition_id = $1
              AND (starts_on IS NULL OR starts_on <= $2)
              AND (ends_on IS NULL OR ends_on >= $2)
              AND status IN ('active', 'planned', 'completed')
            ORDER BY
              CASE status WHEN 'active' THEN 0 WHEN 'planned' THEN 1 ELSE 2 END,
              starts_on DESC NULLS LAST,
              id
            LIMIT 1
            "#,
        )
        .bind(competition_id)
        .bind(kickoff_date)
        .fetch_optional(pool)
        .await?;
        if resolved.season_id.is_none() {
            let (season_name, starts_on, ends_on) =
                automatic_season_identity(&season_pattern, kickoff_date)?;
            resolved.season_id = Some(
                sqlx::query_scalar::<_, Uuid>(
                    r#"
                    INSERT INTO football.seasons (
                        id, competition_id, name, starts_on, ends_on, status, metadata
                    ) VALUES (
                        $1, $2, $3, $4, $5, 'active',
                        jsonb_build_object(
                            'auto_created', true,
                            'season_pattern', $6,
                            'competition_timezone', $7,
                            'local_kickoff_date', $8::text
                        )
                    )
                    ON CONFLICT (competition_id, name) DO UPDATE SET
                        starts_on = COALESCE(football.seasons.starts_on, EXCLUDED.starts_on),
                        ends_on = COALESCE(football.seasons.ends_on, EXCLUDED.ends_on),
                        status = CASE
                            WHEN football.seasons.status = 'archived' THEN football.seasons.status
                            ELSE 'active'
                        END,
                        metadata = football.seasons.metadata || EXCLUDED.metadata
                    RETURNING id
                    "#,
                )
                .bind(Uuid::new_v4())
                .bind(competition_id)
                .bind(season_name)
                .bind(starts_on)
                .bind(ends_on)
                .bind(season_pattern)
                .bind(timezone)
                .bind(kickoff_date)
                .fetch_one(pool)
                .await?,
            );
        }
    }
    Ok(resolved)
}

fn automatic_season_identity(
    season_pattern: &str,
    kickoff_date: NaiveDate,
) -> PersistenceResult<(String, NaiveDate, NaiveDate)> {
    let year = kickoff_date.year();
    if season_pattern.eq_ignore_ascii_case("cross_year") {
        let start_year = if kickoff_date.month() >= 7 {
            year
        } else {
            year - 1
        };
        let end_year = start_year + 1;
        let starts_on = NaiveDate::from_ymd_opt(start_year, 7, 1)
            .ok_or_else(|| PersistenceError::InvalidState("自动赛季开始日期无效".to_string()))?;
        let ends_on = NaiveDate::from_ymd_opt(end_year, 6, 30)
            .ok_or_else(|| PersistenceError::InvalidState("自动赛季结束日期无效".to_string()))?;
        return Ok((
            format!("{start_year}/{}", end_year % 100),
            starts_on,
            ends_on,
        ));
    }
    let starts_on = NaiveDate::from_ymd_opt(year, 1, 1)
        .ok_or_else(|| PersistenceError::InvalidState("自动赛季开始日期无效".to_string()))?;
    let ends_on = NaiveDate::from_ymd_opt(year, 12, 31)
        .ok_or_else(|| PersistenceError::InvalidState("自动赛季结束日期无效".to_string()))?;
    Ok((year.to_string(), starts_on, ends_on))
}

async fn validate_match_scope(pool: &sqlx::PgPool, draft: &MatchDraft) -> PersistenceResult<()> {
    if let Some(round_id) = draft.round_id {
        let row = sqlx::query(
            r#"
            SELECT round.stage_id, stage.season_id, season.competition_id
            FROM football.rounds round
            JOIN football.competition_stages stage ON stage.id = round.stage_id
            JOIN football.seasons season ON season.id = stage.season_id
            WHERE round.id = $1
            "#,
        )
        .bind(round_id)
        .fetch_one(pool)
        .await?;
        let stage_id: Uuid = row.try_get("stage_id")?;
        let season_id: Uuid = row.try_get("season_id")?;
        let competition_id: Uuid = row.try_get("competition_id")?;
        if draft.stage_id.is_some_and(|value| value != stage_id)
            || draft.season_id.is_some_and(|value| value != season_id)
            || draft
                .competition_id
                .is_some_and(|value| value != competition_id)
        {
            return Err(PersistenceError::InvalidState(
                "比赛轮次、阶段、赛季或赛事层级不一致".to_string(),
            ));
        }
    } else if let Some(stage_id) = draft.stage_id {
        let row = sqlx::query(
            r#"
            SELECT stage.season_id, season.competition_id
            FROM football.competition_stages stage
            JOIN football.seasons season ON season.id = stage.season_id
            WHERE stage.id = $1
            "#,
        )
        .bind(stage_id)
        .fetch_one(pool)
        .await?;
        let season_id: Uuid = row.try_get("season_id")?;
        let competition_id: Uuid = row.try_get("competition_id")?;
        if draft.season_id.is_some_and(|value| value != season_id)
            || draft
                .competition_id
                .is_some_and(|value| value != competition_id)
        {
            return Err(PersistenceError::InvalidState(
                "比赛阶段、赛季或赛事层级不一致".to_string(),
            ));
        }
    } else if let Some(season_id) = draft.season_id {
        let competition_id: Uuid =
            sqlx::query_scalar("SELECT competition_id FROM football.seasons WHERE id = $1")
                .bind(season_id)
                .fetch_one(pool)
                .await?;
        if draft
            .competition_id
            .is_some_and(|value| value != competition_id)
        {
            return Err(PersistenceError::InvalidState(
                "比赛赛季不属于所选赛事".to_string(),
            ));
        }
    }
    Ok(())
}

async fn audit_in_tx(
    tx: &mut Transaction<'_, Postgres>,
    event_type: &str,
    entity_type: &str,
    entity_id: String,
    payload: Value,
) -> PersistenceResult<()> {
    sqlx::query(
        r#"
        INSERT INTO audit.events (id, event_type, entity_type, entity_id, payload)
        VALUES ($1, $2, $3, $4, $5)
        "#,
    )
    .bind(Uuid::new_v4())
    .bind(event_type)
    .bind(entity_type)
    .bind(entity_id)
    .bind(payload)
    .execute(&mut **tx)
    .await?;
    Ok(())
}

fn normalize_name(value: &str) -> String {
    value
        .trim()
        .to_lowercase()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn preferred_foot(value: &str) -> PersistenceResult<PreferredFoot> {
    match value {
        "left" => Ok(PreferredFoot::Left),
        "right" => Ok(PreferredFoot::Right),
        "both" => Ok(PreferredFoot::Both),
        "unknown" => Ok(PreferredFoot::Unknown),
        other => Err(PersistenceError::InvalidState(format!(
            "未知惯用脚类型：{other}"
        ))),
    }
}

fn player_status(value: &str) -> PersistenceResult<PlayerStatus> {
    match value {
        "active" => Ok(PlayerStatus::Active),
        "inactive" => Ok(PlayerStatus::Inactive),
        "retired" => Ok(PlayerStatus::Retired),
        "unknown" => Ok(PlayerStatus::Unknown),
        other => Err(PersistenceError::InvalidState(format!(
            "未知球员状态：{other}"
        ))),
    }
}

fn availability_status(value: &str) -> PersistenceResult<AvailabilityStatus> {
    match value {
        "available" => Ok(AvailabilityStatus::Available),
        "doubtful" => Ok(AvailabilityStatus::Doubtful),
        "unavailable" => Ok(AvailabilityStatus::Unavailable),
        "injured" => Ok(AvailabilityStatus::Injured),
        "suspended" => Ok(AvailabilityStatus::Suspended),
        "rested" => Ok(AvailabilityStatus::Rested),
        "returning" => Ok(AvailabilityStatus::Returning),
        "unknown" => Ok(AvailabilityStatus::Unknown),
        other => Err(PersistenceError::InvalidState(format!(
            "未知球员可用性：{other}"
        ))),
    }
}

fn match_status(value: &str) -> PersistenceResult<MatchStatus> {
    match value {
        "scheduled" => Ok(MatchStatus::Scheduled),
        "live" => Ok(MatchStatus::Live),
        "finished" => Ok(MatchStatus::Finished),
        "postponed" => Ok(MatchStatus::Postponed),
        "cancelled" => Ok(MatchStatus::Cancelled),
        other => Err(PersistenceError::InvalidState(format!(
            "未知比赛状态：{other}"
        ))),
    }
}

struct ValidatedLineupDraft {
    snapshot_type: String,
    starters: usize,
}

fn validate_lineup_draft(draft: &LineupDraft) -> PersistenceResult<ValidatedLineupDraft> {
    if !(1..=30).contains(&draft.players.len()) {
        return Err(PersistenceError::InvalidState(
            "阵容球员数量必须位于 1–30".to_string(),
        ));
    }
    let unique_players: HashSet<Uuid> = draft
        .players
        .iter()
        .map(|player| player.player_id)
        .collect();
    if unique_players.len() != draft.players.len() {
        return Err(PersistenceError::InvalidState(
            "同一阵容中存在重复球员".to_string(),
        ));
    }
    let starters = draft
        .players
        .iter()
        .filter(|player| player.is_starter)
        .count();
    if starters != 11 {
        return Err(PersistenceError::InvalidState(format!(
            "正式阵容必须恰好 11 名首发，当前为 {starters} 名",
        )));
    }
    if draft
        .quality_score
        .is_some_and(|value| !(0.0..=1.0).contains(&value))
    {
        return Err(PersistenceError::InvalidState(
            "阵容质量分必须位于 0–1".to_string(),
        ));
    }
    let snapshot_type =
        crate::lineup_chain::normalize_lineup_snapshot_type(&draft.snapshot_type)?.to_string();
    for player in &draft.players {
        if player
            .shirt_number
            .is_some_and(|number| !(0..=99).contains(&number))
        {
            return Err(PersistenceError::InvalidState(
                "阵容球衣号码必须位于 0–99".to_string(),
            ));
        }
        if player
            .expected_minutes
            .is_some_and(|minutes| !(0..=150).contains(&minutes))
            || player
                .actual_minutes
                .is_some_and(|minutes| !(0..=150).contains(&minutes))
        {
            return Err(PersistenceError::InvalidState(
                "阵容分钟数必须位于 0–150".to_string(),
            ));
        }
        if player.sequence_no < 0 {
            return Err(PersistenceError::InvalidState(
                "阵容排序号不能为负数".to_string(),
            ));
        }
        if player
            .bench_order
            .is_some_and(|value| !(1..=99).contains(&value))
        {
            return Err(PersistenceError::InvalidState(
                "替补顺序必须位于 1–99".to_string(),
            ));
        }
        if player
            .starting_probability
            .is_some_and(|value| !(0.0..=1.0).contains(&value))
        {
            return Err(PersistenceError::InvalidState(
                "首发概率必须位于 0–1".to_string(),
            ));
        }
    }
    Ok(ValidatedLineupDraft {
        snapshot_type,
        starters,
    })
}

async fn insert_lineup_in_tx(
    tx: &mut Transaction<'_, Postgres>,
    draft: &LineupDraft,
    validated: &ValidatedLineupDraft,
) -> PersistenceResult<Uuid> {
    let valid_team: bool = sqlx::query_scalar(
        r#"
        SELECT EXISTS (
            SELECT 1 FROM football.matches fixture
            WHERE fixture.id = $1
              AND (fixture.home_team_id = $2 OR fixture.away_team_id = $2)
        )
        "#,
    )
    .bind(draft.match_id)
    .bind(draft.team_id)
    .fetch_one(&mut **tx)
    .await?;
    if !valid_team {
        return Err(PersistenceError::InvalidState(
            "阵容球队不是该场比赛的参赛队".to_string(),
        ));
    }

    let requested_formation = draft
        .formation
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty());
    let formation_row = if let Some(formation_id) = draft.formation_id {
        sqlx::query("SELECT id, code FROM football.formations WHERE id=$1 AND is_active")
            .bind(formation_id)
            .fetch_optional(&mut **tx)
            .await?
    } else if let Some(formation) = requested_formation {
        sqlx::query(
            r#"
            SELECT id, code FROM football.formations
            WHERE is_active
              AND regexp_replace(lower(trim(code)), '\\s+', '', 'g') =
                  regexp_replace(lower(trim($1)), '\\s+', '', 'g')
            LIMIT 1
            "#,
        )
        .bind(formation)
        .fetch_optional(&mut **tx)
        .await?
    } else {
        None
    };
    if draft.formation_id.is_some() && formation_row.is_none() {
        return Err(PersistenceError::InvalidState(
            "所选阵型不存在或已停用".to_string(),
        ));
    }
    let resolved_formation_id = formation_row
        .as_ref()
        .map(|row| row.try_get::<Uuid, _>("id"))
        .transpose()?;
    let formation_text = formation_row
        .as_ref()
        .and_then(|row| row.try_get::<String, _>("code").ok())
        .or_else(|| requested_formation.map(str::to_string));

    let supersedes_lineup_id: Option<Uuid> = sqlx::query_scalar(
        r#"
        SELECT id FROM football.lineups
        WHERE match_id=$1 AND team_id=$2 AND snapshot_type=$3
          AND lineup_type=$4 AND status='active'
        ORDER BY captured_at DESC, created_at DESC, id DESC
        LIMIT 1
        "#,
    )
    .bind(draft.match_id)
    .bind(draft.team_id)
    .bind(&validated.snapshot_type)
    .bind(draft.lineup_type.as_str())
    .fetch_optional(&mut **tx)
    .await?;

    sqlx::query(
        r#"
        UPDATE football.lineups
        SET status='superseded', updated_at=now()
        WHERE match_id=$1 AND team_id=$2 AND snapshot_type=$3
          AND lineup_type=$4 AND status='active'
        "#,
    )
    .bind(draft.match_id)
    .bind(draft.team_id)
    .bind(&validated.snapshot_type)
    .bind(draft.lineup_type.as_str())
    .execute(&mut **tx)
    .await?;

    let lineup_id = Uuid::new_v4();
    sqlx::query(
        r#"
        INSERT INTO football.lineups (
            id, match_id, team_id, lineup_type, snapshot_type,
            formation, formation_id, coach_id, captured_at,
            source_document_id, source_urls, supersedes_lineup_id,
            status, quality_score, metadata
        ) VALUES (
            $1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,
            'active',$13,$14
        )
        "#,
    )
    .bind(lineup_id)
    .bind(draft.match_id)
    .bind(draft.team_id)
    .bind(draft.lineup_type.as_str())
    .bind(&validated.snapshot_type)
    .bind(formation_text)
    .bind(resolved_formation_id)
    .bind(draft.coach_id)
    .bind(draft.captured_at)
    .bind(draft.source_document_id)
    .bind(&draft.source_urls)
    .bind(supersedes_lineup_id)
    .bind(draft.quality_score)
    .bind(&draft.metadata)
    .execute(&mut **tx)
    .await?;

    for player in &draft.players {
        let position_code = player
            .position_code
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_uppercase);
        let inherited_role = resolve_default_tactical_role_in_tx(
            tx,
            player.player_id,
            position_code.as_deref(),
            draft.captured_at.date_naive(),
        )
        .await?;
        let role_resolution =
            resolve_tactical_role(player.role_code.as_deref(), inherited_role.as_ref());
        let player_metadata = metadata_with_role_resolution(&player.metadata, &role_resolution);
        sqlx::query(
            r#"
            INSERT INTO football.lineup_players (
                lineup_id, player_id, position_code, role_code, is_starter,
                shirt_number, expected_minutes, actual_minutes, sequence_no,
                bench_order, availability_status, starting_probability,
                membership_override, source_urls, metadata
            ) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15)
            "#,
        )
        .bind(lineup_id)
        .bind(player.player_id)
        .bind(position_code)
        .bind(role_resolution.role_code.as_deref())
        .bind(player.is_starter)
        .bind(player.shirt_number)
        .bind(player.expected_minutes)
        .bind(player.actual_minutes)
        .bind(player.sequence_no)
        .bind(player.bench_order)
        .bind(player.availability_status.map(AvailabilityStatus::as_str))
        .bind(player.starting_probability)
        .bind(player.membership_override)
        .bind(&player.source_urls)
        .bind(player_metadata)
        .execute(&mut **tx)
        .await?;
    }
    crate::lineup_chain::refresh_lineup_validation_in_tx(tx, lineup_id).await?;
    audit_in_tx(
        tx,
        "lineup_created",
        "lineup",
        lineup_id.to_string(),
        json!({
            "match_id": draft.match_id,
            "team_id": draft.team_id,
            "lineup_type": draft.lineup_type.as_str(),
            "snapshot_type": validated.snapshot_type,
            "player_count": draft.players.len(),
            "starter_count": validated.starters,
        }),
    )
    .await?;
    Ok(lineup_id)
}

fn lineup_type(value: &str) -> PersistenceResult<LineupType> {
    match value {
        "expected" => Ok(LineupType::Expected),
        "confirmed" => Ok(LineupType::Confirmed),
        "actual" => Ok(LineupType::Actual),
        other => Err(PersistenceError::InvalidState(format!(
            "未知阵容类型：{other}"
        ))),
    }
}

fn team_record_from_row(row: &sqlx::postgres::PgRow) -> PersistenceResult<TeamRecord> {
    Ok(TeamRecord {
        id: row.try_get("id")?,
        canonical_name: row.try_get("canonical_name")?,
        normalized_name: row.try_get("normalized_name")?,
        country_code: row.try_get("country_code")?,
        is_active: row.try_get("is_active")?,
        created_at: row.try_get("created_at")?,
    })
}

fn team_option_from_row(row: &sqlx::postgres::PgRow) -> PersistenceResult<TeamOption> {
    Ok(TeamOption {
        id: row.try_get("id")?,
        canonical_name: row.try_get("canonical_name")?,
        country_code: row.try_get("country_code")?,
        team_type: row.try_get("team_type")?,
    })
}

fn data_provider_from_row(row: &sqlx::postgres::PgRow) -> PersistenceResult<DataProviderRecord> {
    Ok(DataProviderRecord {
        id: row.try_get("id")?,
        code: row.try_get("code")?,
        name: row.try_get("name")?,
        provider_type: row.try_get("provider_type")?,
        base_url: row.try_get("base_url")?,
        is_active: row.try_get("is_active")?,
    })
}

fn player_record_from_row(row: &sqlx::postgres::PgRow) -> PersistenceResult<PlayerRecord> {
    let foot: String = row.try_get("preferred_foot")?;
    let status: String = row.try_get("status")?;
    Ok(PlayerRecord {
        id: row.try_get("id")?,
        canonical_name: row.try_get("canonical_name")?,
        normalized_name: row.try_get("normalized_name")?,
        date_of_birth: row.try_get("date_of_birth")?,
        nationality_code: row.try_get("nationality_code")?,
        preferred_foot: preferred_foot(&foot)?,
        height_cm: row.try_get("height_cm")?,
        status: player_status(&status)?,
        created_at: row.try_get("created_at")?,
    })
}

fn player_list_item_from_row(row: &sqlx::postgres::PgRow) -> PersistenceResult<PlayerListItem> {
    let foot: String = row.try_get("preferred_foot")?;
    let status: String = row.try_get("status")?;
    let availability: Option<String> = row.try_get("availability_status")?;
    Ok(PlayerListItem {
        id: row.try_get("id")?,
        canonical_name: row.try_get("canonical_name")?,
        localized_name: row.try_get("localized_name")?,
        alternate_name: row.try_get("alternate_name")?,
        normalized_name: row.try_get("normalized_name")?,
        date_of_birth: row.try_get("date_of_birth")?,
        nationality_code: row.try_get("nationality_code")?,
        preferred_foot: preferred_foot(&foot)?,
        status: player_status(&status)?,
        current_team_id: row.try_get("current_team_id")?,
        current_team_name: row.try_get("current_team_name")?,
        primary_position_code: row.try_get("primary_position_code")?,
        primary_role_code: row.try_get("primary_role_code")?,
        position_role_map: row.try_get("position_role_map")?,
        availability_status: availability
            .as_deref()
            .map(availability_status)
            .transpose()?,
        availability_reason: row.try_get("availability_reason")?,
        availability_confidence: row.try_get("availability_confidence")?,
        availability_valid_to: row.try_get("availability_valid_to")?,
        availability_competition_name: row.try_get("availability_competition_name")?,
        ability_average: row.try_get("ability_average")?,
        ability_confidence: row.try_get("ability_confidence")?,
        ability_dimension_count: row.try_get("ability_dimension_count")?,
    })
}

fn player_name_from_row(row: &sqlx::postgres::PgRow) -> PersistenceResult<PlayerNameRecord> {
    Ok(PlayerNameRecord {
        id: row.try_get("id")?,
        player_id: row.try_get("player_id")?,
        name: row.try_get("name")?,
        normalized_name: row.try_get("normalized_name")?,
        language_code: row.try_get("language_code")?,
        is_primary: row.try_get("is_primary")?,
        valid_from: row.try_get("valid_from")?,
        valid_to: row.try_get("valid_to")?,
    })
}

fn player_position_from_row(
    row: &sqlx::postgres::PgRow,
) -> PersistenceResult<PlayerPositionRecord> {
    Ok(PlayerPositionRecord {
        id: row.try_get("id")?,
        player_id: row.try_get("player_id")?,
        position_code: row.try_get("position_code")?,
        position_name: row.try_get("position_name")?,
        position_group: row.try_get("position_group")?,
        proficiency: row.try_get("proficiency")?,
        default_role_code: row.try_get("default_role_code")?,
        is_primary: row.try_get("is_primary")?,
        valid_from: row.try_get("valid_from")?,
        valid_to: row.try_get("valid_to")?,
    })
}

fn player_team_period_from_row(
    row: &sqlx::postgres::PgRow,
) -> PersistenceResult<PlayerTeamPeriodRecord> {
    Ok(PlayerTeamPeriodRecord {
        id: row.try_get("id")?,
        player_id: row.try_get("player_id")?,
        team_id: row.try_get("team_id")?,
        team_name: row.try_get("team_name")?,
        season_id: row.try_get("season_id")?,
        season_name: row.try_get("season_name")?,
        squad_number: row.try_get("squad_number")?,
        valid_from: row.try_get("valid_from")?,
        valid_to: row.try_get("valid_to")?,
        registration_status: row.try_get("registration_status")?,
    })
}

fn player_availability_from_row(
    row: &sqlx::postgres::PgRow,
) -> PersistenceResult<PlayerAvailabilityRecord> {
    let status: String = row.try_get("status")?;
    Ok(PlayerAvailabilityRecord {
        id: row.try_get("id")?,
        player_id: row.try_get("player_id")?,
        team_id: row.try_get("team_id")?,
        team_name: row.try_get("team_name")?,
        competition_id: row.try_get("competition_id")?,
        status: availability_status(&status)?,
        reason: row.try_get("reason")?,
        confidence: row.try_get("confidence")?,
        valid_from: row.try_get("valid_from")?,
        valid_to: row.try_get("valid_to")?,
        created_at: row.try_get("created_at")?,
    })
}

fn player_ability_observation_from_row(
    row: &sqlx::postgres::PgRow,
) -> PersistenceResult<PlayerAbilityObservationRecord> {
    Ok(PlayerAbilityObservationRecord {
        id: row.try_get("id")?,
        player_id: row.try_get("player_id")?,
        dimension_code: row.try_get("dimension_code")?,
        dimension_name: row.try_get("dimension_name")?,
        context_type: row.try_get("context_type")?,
        context_id: row.try_get("context_id")?,
        value: row.try_get("value")?,
        confidence: row.try_get("confidence")?,
        sample_size: row.try_get("sample_size")?,
        observed_at: row.try_get("observed_at")?,
        effective_from: row.try_get("effective_from")?,
        effective_to: row.try_get("effective_to")?,
        calculation_version: row.try_get("calculation_version")?,
    })
}

fn player_ability_profile_from_row(
    row: &sqlx::postgres::PgRow,
) -> PersistenceResult<PlayerAbilityProfile> {
    Ok(PlayerAbilityProfile {
        player_id: row.try_get("player_id")?,
        abilities: row.try_get("abilities")?,
        average_value: row.try_get("average_value")?,
        average_confidence: row.try_get("average_confidence")?,
        dimension_count: row.try_get("dimension_count")?,
        latest_observed_at: row.try_get("latest_observed_at")?,
        next_expiry_at: row.try_get("next_expiry_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}

fn external_entity_id_from_row(
    row: &sqlx::postgres::PgRow,
) -> PersistenceResult<ExternalEntityIdRecord> {
    Ok(ExternalEntityIdRecord {
        id: row.try_get("id")?,
        provider_id: row.try_get("provider_id")?,
        provider_name: row.try_get("provider_name")?,
        entity_type: row.try_get("entity_type")?,
        entity_id: row.try_get("entity_id")?,
        external_id: row.try_get("external_id")?,
        metadata: row.try_get("metadata")?,
    })
}

fn match_record_from_row(row: &sqlx::postgres::PgRow) -> PersistenceResult<MatchRecord> {
    let status: String = row.try_get("status")?;
    Ok(MatchRecord {
        id: row.try_get("id")?,
        external_key: row.try_get("external_key")?,
        competition_id: row.try_get("competition_id")?,
        competition_name: row.try_get("competition_name")?,
        season_id: row.try_get("season_id")?,
        stage_id: row.try_get("stage_id")?,
        round_id: row.try_get("round_id")?,
        home_team_id: row.try_get("home_team_id")?,
        home_team_name: row.try_get("home_team_name")?,
        away_team_id: row.try_get("away_team_id")?,
        away_team_name: row.try_get("away_team_name")?,
        kickoff_time: row.try_get("kickoff_time")?,
        status: match_status(&status)?,
        venue: row.try_get("venue")?,
    })
}

pub(crate) fn lineup_player_from_row(
    row: &sqlx::postgres::PgRow,
) -> PersistenceResult<LineupPlayerRecord> {
    let availability: Option<String> = row.try_get("availability_status")?;
    Ok(LineupPlayerRecord {
        player_id: row.try_get("player_id")?,
        player_name: row.try_get("player_name")?,
        position_code: row.try_get("position_code")?,
        role_code: row.try_get("role_code")?,
        role_origin: row.try_get("role_origin")?,
        role_source_position_code: row.try_get("role_source_position_code")?,
        is_starter: row.try_get("is_starter")?,
        shirt_number: row.try_get("shirt_number")?,
        expected_minutes: row.try_get("expected_minutes")?,
        actual_minutes: row.try_get("actual_minutes")?,
        sequence_no: row.try_get("sequence_no")?,
        bench_order: row.try_get("bench_order")?,
        availability_status: availability
            .as_deref()
            .map(availability_status)
            .transpose()?,
        starting_probability: row.try_get("starting_probability")?,
        membership_override: row.try_get("membership_override")?,
        source_urls: row.try_get("source_urls")?,
        validation_warning: row.try_get("validation_warning")?,
    })
}

pub(crate) fn lineup_record_from_row(
    row: &sqlx::postgres::PgRow,
    players: Vec<LineupPlayerRecord>,
) -> PersistenceResult<LineupRecord> {
    let lineup_type_value: String = row.try_get("lineup_type")?;
    Ok(LineupRecord {
        id: row.try_get("id")?,
        match_id: row.try_get("match_id")?,
        match_key: row.try_get("match_key")?,
        team_id: row.try_get("team_id")?,
        team_name: row.try_get("team_name")?,
        lineup_type: lineup_type(&lineup_type_value)?,
        snapshot_type: row.try_get("snapshot_type")?,
        formation: row.try_get("formation")?,
        formation_id: row.try_get("formation_id")?,
        formation_code: row.try_get("formation_code")?,
        formation_name: row.try_get("formation_name")?,
        coach_id: row.try_get("coach_id")?,
        coach_name: row.try_get("coach_name")?,
        captured_at: row.try_get("captured_at")?,
        status: row.try_get("status")?,
        quality_score: row.try_get("quality_score")?,
        source_urls: row.try_get("source_urls")?,
        supersedes_lineup_id: row.try_get("supersedes_lineup_id")?,
        model_validation_status: row.try_get("model_validation_status")?,
        model_eligible: row.try_get("model_eligible")?,
        validation_errors: serde_json::from_value(row.try_get("validation_errors")?)?,
        validation_warnings: serde_json::from_value(row.try_get("validation_warnings")?)?,
        player_count: row.try_get("player_count")?,
        starter_count: row.try_get("starter_count")?,
        players,
    })
}

fn position_reference_from_row(
    row: &sqlx::postgres::PgRow,
) -> PersistenceResult<PositionReference> {
    Ok(PositionReference {
        code: row.try_get("code")?,
        name: row.try_get("name")?,
        position_group: row.try_get("position_group")?,
        sort_order: row.try_get("sort_order")?,
    })
}

fn ability_dimension_from_row(
    row: &sqlx::postgres::PgRow,
) -> PersistenceResult<AbilityDimensionRecord> {
    Ok(AbilityDimensionRecord {
        code: row.try_get("code")?,
        name: row.try_get("name")?,
        category: row.try_get("category")?,
        minimum_value: row.try_get("minimum_value")?,
        maximum_value: row.try_get("maximum_value")?,
        description: row.try_get("description")?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_name_collapses_case_and_spacing() {
        assert_eq!(normalize_name("  Son   Heung-Min  "), "son heung-min");
    }

    #[test]
    fn persisted_enums_round_trip() {
        assert_eq!(preferred_foot("left").unwrap(), PreferredFoot::Left);
        assert_eq!(player_status("active").unwrap(), PlayerStatus::Active);
        assert_eq!(
            availability_status("returning").unwrap(),
            AvailabilityStatus::Returning
        );
        assert_eq!(match_status("finished").unwrap(), MatchStatus::Finished);
        assert_eq!(lineup_type("confirmed").unwrap(), LineupType::Confirmed);
    }

    #[test]
    fn unknown_persisted_enum_is_rejected() {
        assert!(preferred_foot("ambidextrous").is_err());
        assert!(availability_status("missing").is_err());
        assert!(lineup_type("draft").is_err());
    }
}
