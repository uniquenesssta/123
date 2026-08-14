use super::{map_season_row, SeasonRow};
use crate::{PersistenceResult, PostgresStore};
use football_domain::SeasonRecord;

impl PostgresStore {
    pub async fn list_seasons(&self) -> PersistenceResult<Vec<SeasonRecord>> {
        let rows = sqlx::query_as::<_, SeasonRow>(
            r#"
            SELECT
                s.id, s.competition_id, c.name AS competition_name,
                s.name, s.starts_on, s.ends_on, s.status
            FROM football.seasons s
            JOIN football.competitions c ON c.id = s.competition_id
            WHERE c.is_active = true
            ORDER BY c.name, s.starts_on DESC NULLS LAST, s.name
            "#,
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.into_iter().map(map_season_row).collect())
    }
}
