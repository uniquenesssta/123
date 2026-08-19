use super::{option_mapper::map_team_option_row, option_row::TeamOptionRow};
use crate::{
    adapters::catalog::global_search::{push_name_search, NameSearch, NameSearchColumns},
    PersistenceResult, PostgresStore,
};
use football_domain::TeamOption;
use sqlx::{Postgres, QueryBuilder};

impl PostgresStore {
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
        let rows = builder
            .build_query_as::<TeamOptionRow>()
            .fetch_all(&self.pool)
            .await?;
        Ok(rows.into_iter().map(map_team_option_row).collect())
    }
}
