use super::{map_team_record, TeamRecordRow};
use crate::{PersistenceError, PersistenceResult};
use football_domain::TeamRecord;
use sqlx::PgPool;
use uuid::Uuid;

pub(super) async fn read_team_record(
    pool: &PgPool,
    team_id: Uuid,
) -> PersistenceResult<TeamRecord> {
    let row = sqlx::query_as::<_, TeamRecordRow>(
        r#"
        SELECT id, canonical_name, normalized_name, country_code, is_active, created_at
        FROM football.teams
        WHERE id = $1
        "#,
    )
    .bind(team_id)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| PersistenceError::InvalidState("球队不存在".to_string()))?;
    map_team_record(row)
}
