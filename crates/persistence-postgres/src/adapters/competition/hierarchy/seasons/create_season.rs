use super::read_season;
use crate::{PersistenceResult, PostgresStore};
use football_domain::{SeasonDraft, SeasonRecord};
use uuid::Uuid;

impl PostgresStore {
    pub async fn create_season(&self, draft: &SeasonDraft) -> PersistenceResult<SeasonRecord> {
        let id = Uuid::new_v4();
        sqlx::query(
            r#"
            INSERT INTO football.seasons (
                id, competition_id, name, starts_on, ends_on, status, metadata
            ) VALUES ($1, $2, $3, $4, $5, $6, $7)
            "#,
        )
        .bind(id)
        .bind(draft.competition_id)
        .bind(draft.name.trim())
        .bind(draft.starts_on)
        .bind(draft.ends_on)
        .bind(draft.status.trim())
        .bind(&draft.metadata)
        .execute(&self.pool)
        .await?;
        read_season(self, id).await
    }
}
