use super::{map_season_row, SeasonRow};
use crate::{PersistenceResult, PostgresStore};
use football_domain::SeasonRecord;
use uuid::Uuid;

pub(super) async fn read_season(
    store: &PostgresStore,
    id: Uuid,
) -> PersistenceResult<SeasonRecord> {
    let row = sqlx::query_as::<_, SeasonRow>(
        r#"
        SELECT
            s.id, s.competition_id, c.name AS competition_name,
            s.name, s.starts_on, s.ends_on, s.status
        FROM football.seasons s
        JOIN football.competitions c ON c.id = s.competition_id
        WHERE s.id = $1
        "#,
    )
    .bind(id)
    .fetch_one(&store.pool)
    .await?;
    Ok(map_season_row(row))
}
