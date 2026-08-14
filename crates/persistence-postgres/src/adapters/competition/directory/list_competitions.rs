use super::super::detail::{map_competition_row, CompetitionRow};
use crate::{PersistenceResult, PostgresStore};
use football_domain::CompetitionRecord;

impl PostgresStore {
    pub async fn list_competitions(&self) -> PersistenceResult<Vec<CompetitionRecord>> {
        let rows = sqlx::query_as::<_, CompetitionRow>(
            r#"
            SELECT id, code, name, country_code, timezone,
                   competition_kind, is_active, metadata, created_at
            FROM football.competitions
            WHERE is_active = true
            ORDER BY COALESCE((metadata->>'sort_order')::integer, 999999), name, code
            "#,
        )
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter().map(map_competition_row).collect()
    }
}
