use super::{map_round_row, RoundRow};
use crate::{PersistenceResult, PostgresStore};
use football_domain::RoundRecord;
use uuid::Uuid;

pub(super) async fn read_round(store: &PostgresStore, id: Uuid) -> PersistenceResult<RoundRecord> {
    let row = sqlx::query_as::<_, RoundRow>(
        r#"
        SELECT
            r.id, r.stage_id, st.name AS stage_name,
            r.code, r.name, r.sequence_no, r.starts_at, r.ends_at
        FROM football.rounds r
        JOIN football.competition_stages st ON st.id = r.stage_id
        WHERE r.id = $1
        "#,
    )
    .bind(id)
    .fetch_one(&store.pool)
    .await?;
    Ok(map_round_row(row))
}
