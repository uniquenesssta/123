use super::mapper::{coach_reference_from_row, player_reference_from_row, team_reference_from_row};
use crate::{
    adapters::catalog::global_search::{push_name_search, NameSearch, NameSearchColumns},
    PersistenceResult,
};
use football_domain::{EntityReferenceQuery, EntityReferenceRecord};
use sqlx::{PgPool, Postgres, QueryBuilder};

pub(super) async fn team_references(
    pool: &PgPool,
    query: &EntityReferenceQuery,
) -> PersistenceResult<Vec<EntityReferenceRecord>> {
    let mut builder: QueryBuilder<Postgres> = QueryBuilder::new(
        r#"
        SELECT team.id, team.canonical_name, team.normalized_name, team.country_code,
               team.is_active,
               COALESCE((SELECT array_agg(alias.name ORDER BY alias.name) FROM football.team_names alias WHERE alias.team_id=team.id), ARRAY[]::text[]) AS aliases,
               COALESCE((SELECT array_agg(external.external_id ORDER BY external.external_id) FROM football.external_entity_ids external WHERE external.entity_type='team' AND external.entity_id=team.id), ARRAY[]::text[]) AS external_ids
        FROM football.teams team
        WHERE 1=1
        "#,
    );
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
    if query.active_only {
        builder.push(" AND team.is_active");
    }
    builder.push(" ORDER BY team.normalized_name, team.id LIMIT ");
    builder.push_bind(i64::from(query.limit.clamp(1, 500)));
    builder
        .build()
        .fetch_all(pool)
        .await?
        .iter()
        .map(team_reference_from_row)
        .collect()
}

pub(super) async fn player_references(
    pool: &PgPool,
    query: &EntityReferenceQuery,
) -> PersistenceResult<Vec<EntityReferenceRecord>> {
    let mut builder: QueryBuilder<Postgres> = QueryBuilder::new(
        r#"
        SELECT player.id, player.canonical_name, player.normalized_name,
               player.date_of_birth, player.nationality_code, player.status,
               COALESCE((SELECT array_agg(alias.name ORDER BY alias.name) FROM football.player_names alias WHERE alias.player_id=player.id), ARRAY[]::text[]) AS aliases,
               COALESCE((SELECT array_agg(external.external_id ORDER BY external.external_id) FROM football.external_entity_ids external WHERE external.entity_type='player' AND external.entity_id=player.id), ARRAY[]::text[]) AS external_ids
        FROM football.players player
        WHERE 1=1
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
    if query.active_only {
        builder.push(" AND player.status='active'");
    }
    builder.push(" ORDER BY player.normalized_name, player.id LIMIT ");
    builder.push_bind(i64::from(query.limit.clamp(1, 500)));
    builder
        .build()
        .fetch_all(pool)
        .await?
        .iter()
        .map(player_reference_from_row)
        .collect()
}

pub(super) async fn coach_references(
    pool: &PgPool,
    query: &EntityReferenceQuery,
) -> PersistenceResult<Vec<EntityReferenceRecord>> {
    let mut builder: QueryBuilder<Postgres> = QueryBuilder::new(
        r#"
        SELECT coach.id, coach.canonical_name, coach.normalized_name,
               coach.nationality_code, coach.status,
               COALESCE((SELECT array_agg(alias.name ORDER BY alias.name) FROM football.coach_names alias WHERE alias.coach_id=coach.id), ARRAY[]::text[]) AS aliases,
               COALESCE((SELECT array_agg(external.external_id ORDER BY external.external_id) FROM football.external_entity_ids external WHERE external.entity_type='coach' AND external.entity_id=coach.id), ARRAY[]::text[]) AS external_ids
        FROM football.coaches coach
        WHERE 1=1
        "#,
    );
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
    if query.active_only {
        builder.push(" AND coach.status='active'");
    }
    builder.push(" ORDER BY coach.normalized_name, coach.id LIMIT ");
    builder.push_bind(i64::from(query.limit.clamp(1, 500)));
    builder
        .build()
        .fetch_all(pool)
        .await?
        .iter()
        .map(coach_reference_from_row)
        .collect()
}
