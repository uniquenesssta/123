use super::{map_competition_row, CompetitionRow};
use crate::{PersistenceResult, PostgresStore};
use football_domain::CompetitionRecord;
use uuid::Uuid;

impl PostgresStore {
    pub async fn read_competition(&self, id: Uuid) -> PersistenceResult<CompetitionRecord> {
        let row = sqlx::query_as::<_, CompetitionRow>(
            r#"
            SELECT id, code, name, country_code, timezone,
                   competition_kind, is_active, metadata, created_at
            FROM football.competitions
            WHERE id = $1 AND is_active = true
            "#,
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await?;
        map_competition_row(row)
    }
}
