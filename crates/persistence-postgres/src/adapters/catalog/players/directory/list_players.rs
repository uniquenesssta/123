use super::{list_mapper::map_player_list_item, list_row::PlayerListRow};
use crate::{
    name_search::{push_name_search, NameSearch, NameSearchColumns},
    PersistenceError, PersistenceResult, PostgresStore,
};
use football_domain::{PlayerListItem, PlayerListPage, PlayerListQuery};
use sqlx::{Postgres, QueryBuilder};

impl PostgresStore {
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

        let rows = builder
            .build_query_as::<PlayerListRow>()
            .fetch_all(&self.pool)
            .await?;
        let has_more = rows.len() > limit as usize;
        let mut items: Vec<PlayerListItem> = rows
            .into_iter()
            .take(limit as usize)
            .map(map_player_list_item)
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
}
