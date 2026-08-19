use crate::{
    adapters::catalog::coaches::mapping::coach_list_item_from_row,
    adapters::catalog::global_search::{push_name_search, NameSearch, NameSearchColumns},
    PersistenceResult, PostgresStore,
};
use football_domain::{CoachListItem, CoachListQuery};
use sqlx::{Postgres, QueryBuilder};
impl PostgresStore {
    pub async fn list_coaches(
        &self,
        query: &CoachListQuery,
    ) -> PersistenceResult<Vec<CoachListItem>> {
        let mut builder: QueryBuilder<Postgres> = QueryBuilder::new(
            r#"
                SELECT coach.id, coach.canonical_name, coach.nationality_code, coach.status,
                       current_period.team_id AS current_team_id,
                       current_period.team_name AS current_team_name,
                       current_period.role AS current_role
                FROM football.coaches coach
                LEFT JOIN LATERAL (
                    SELECT period.team_id, team.canonical_name AS team_name, period.role
                    FROM football.team_coach_periods period
                    JOIN football.teams team ON team.id = period.team_id
                    WHERE period.coach_id = coach.id
                      AND period.valid_from <= current_date
                      AND (period.valid_to IS NULL OR period.valid_to >= current_date)
                    ORDER BY period.valid_from DESC, period.id DESC
                    LIMIT 1
                ) current_period ON true
                WHERE 1=1
                "#,
        );
        if query.active_only {
            builder.push(" AND coach.status = 'active'");
        }
        if let Some(search) = NameSearch::parse(query.search.as_deref()) {
            push_name_search(
                &mut builder,
                &search,
                NameSearchColumns {
                    primary_normalized: "coach.normalized_name",
                    primary_display: "coach.canonical_name",
                    alias_table: "football.coach_names",
                    alias_owner: "alias.coach_id",
                    owner_id: "coach.id",
                    alias_normalized: "alias.normalized_name",
                    alias_display: "alias.name",
                },
            );
        }
        builder.push(" ORDER BY coach.normalized_name, coach.id LIMIT ");
        builder.push_bind(i64::from(query.limit.clamp(1, 200)));
        builder
            .build()
            .fetch_all(&self.pool)
            .await?
            .iter()
            .map(coach_list_item_from_row)
            .collect()
    }
}
