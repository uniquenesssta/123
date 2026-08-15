use super::{list_mapper::map_team_list_row, list_row::TeamListRow};
use crate::{
    name_search::{push_name_search, NameSearch, NameSearchColumns},
    PersistenceError, PersistenceResult, PostgresStore,
};
use football_domain::{TeamListPage, TeamListQuery};
use sqlx::{Postgres, QueryBuilder};

impl PostgresStore {
    pub async fn list_teams(&self, query: &TeamListQuery) -> PersistenceResult<TeamListPage> {
        if query.cursor_name.is_some() != query.cursor_id.is_some() {
            return Err(PersistenceError::InvalidState(
                "球队分页游标必须同时包含名称和 ID".to_string(),
            ));
        }
        let limit = query.limit.clamp(1, 200);
        let mut builder: QueryBuilder<Postgres> = QueryBuilder::new(
            r#"
            SELECT team.id, team.canonical_name, team.normalized_name, team.country_code,
                   COALESCE(profile.team_type, 'other') AS team_type,
                   current_coach.coach_name AS current_coach_name,
                   team.is_active,
                   COALESCE(squad.current_player_count, 0)::bigint AS current_player_count,
                   COALESCE(squad.unavailable_player_count, 0)::bigint AS unavailable_player_count,
                   squad.squad_ability_average,
                   profile.data_confidence AS profile_confidence
            FROM football.teams team
            LEFT JOIN football.team_profiles profile ON profile.team_id = team.id
            LEFT JOIN LATERAL (
                SELECT coach.canonical_name AS coach_name
                FROM football.team_coach_periods period
                JOIN football.coaches coach ON coach.id = period.coach_id
                WHERE period.team_id = team.id
                  AND period.valid_from <= current_date
                  AND (period.valid_to IS NULL OR period.valid_to >= current_date)
                ORDER BY CASE period.role WHEN 'head_coach' THEN 0 WHEN 'interim_head_coach' THEN 1 ELSE 2 END,
                         period.valid_from DESC, period.id DESC
                LIMIT 1
            ) current_coach ON true
            LEFT JOIN LATERAL (
                SELECT count(*)::bigint AS current_player_count,
                       count(*) FILTER (WHERE availability.status IN ('injured','suspended','doubtful','rested'))::bigint AS unavailable_player_count,
                       avg(ability.average_value) AS squad_ability_average
                FROM football.player_team_periods period
                JOIN football.players player ON player.id = period.player_id
                LEFT JOIN feature.player_ability_profiles ability ON ability.player_id = player.id
                LEFT JOIN LATERAL (
                    SELECT status
                    FROM football.player_availability item
                    WHERE item.player_id = player.id
                      AND item.valid_from <= now()
                      AND (item.valid_to IS NULL OR item.valid_to >= now())
                    ORDER BY item.valid_from DESC, item.created_at DESC
                    LIMIT 1
                ) availability ON true
                WHERE period.team_id = team.id
                  AND period.valid_from <= current_date
                  AND (period.valid_to IS NULL OR period.valid_to >= current_date)
                  AND period.registration_status IN ('registered','loan','trial')
                  AND player.status = 'active'
            ) squad ON true
            WHERE 1 = 1
            "#,
        );
        if query.active_only {
            builder.push(" AND team.is_active");
        }
        if let Some(search) = NameSearch::parse(query.search.as_deref()) {
            push_name_search(
                &mut builder,
                &search,
                NameSearchColumns {
                    primary_normalized: "team.normalized_name",
                    primary_display: "team.canonical_name",
                    alias_table: "football.team_names",
                    alias_owner: "alias.team_id",
                    owner_id: "team.id",
                    alias_normalized: "alias.normalized_name",
                    alias_display: "alias.name",
                },
            );
        }
        if let Some(country) = query
            .country_code
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
        {
            builder.push(" AND upper(COALESCE(team.country_code,'')) = ");
            builder.push_bind(country.to_uppercase());
        }
        if let Some(team_type) = query
            .team_type
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
        {
            builder.push(" AND COALESCE(profile.team_type, 'other') = ");
            builder.push_bind(team_type.to_ascii_lowercase());
        }
        if let (Some(cursor_name), Some(cursor_id)) = (&query.cursor_name, query.cursor_id) {
            builder.push(" AND (team.normalized_name, team.id) > (");
            builder.push_bind(cursor_name);
            builder.push(", ");
            builder.push_bind(cursor_id);
            builder.push(")");
        }
        builder.push(" ORDER BY team.normalized_name, team.id LIMIT ");
        builder.push_bind(i64::from(limit) + 1);
        let rows = builder
            .build_query_as::<TeamListRow>()
            .fetch_all(&self.pool)
            .await?;
        let has_more = rows.len() > limit as usize;
        let items = rows
            .into_iter()
            .take(limit as usize)
            .map(map_team_list_row)
            .collect::<Vec<_>>();
        let next = items.last().filter(|_| has_more);
        Ok(TeamListPage {
            next_cursor_name: next.map(|item| item.normalized_name.clone()),
            next_cursor_id: next.map(|item| item.id),
            items,
            has_more,
        })
    }
}
