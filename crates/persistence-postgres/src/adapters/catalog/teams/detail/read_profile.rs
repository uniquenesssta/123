use super::{profile_mapper::map_team_profile, profile_row::TeamProfileRow};
use crate::PersistenceResult;
use football_domain::TeamProfileRecord;
use sqlx::PgPool;
use uuid::Uuid;

pub(super) async fn read_profile(
    pool: &PgPool,
    team_id: Uuid,
) -> PersistenceResult<Option<TeamProfileRecord>> {
    let row = sqlx::query_as::<_, TeamProfileRow>(
        r#"
        SELECT team_id, short_name, team_type, founded_year, city, stadium, head_coach,
               default_formation, tactical_style, attack_rating, midfield_rating,
               defence_rating, goalkeeper_rating, reputation, data_confidence,
               notes, metadata, updated_at
        FROM football.team_profiles
        WHERE team_id = $1
        "#,
    )
    .bind(team_id)
    .fetch_optional(pool)
    .await?;
    Ok(row.map(map_team_profile))
}
