use super::super::detail::{map_competition_row, CompetitionRow};
use crate::{PersistenceResult, PostgresStore};
use football_domain::{CompetitionDraft, CompetitionRecord};
use uuid::Uuid;

impl PostgresStore {
    pub async fn create_competition(
        &self,
        draft: &CompetitionDraft,
    ) -> PersistenceResult<CompetitionRecord> {
        let id = Uuid::new_v4();
        let row = sqlx::query_as::<_, CompetitionRow>(
            r#"
            INSERT INTO football.competitions (
                id, code, name, country_code, timezone, competition_kind, metadata
            ) VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING id, code, name, country_code, timezone,
                      competition_kind, is_active, metadata, created_at
            "#,
        )
        .bind(id)
        .bind(draft.code.trim())
        .bind(draft.name.trim())
        .bind(draft.country_code.as_deref())
        .bind(draft.timezone.trim())
        .bind(draft.competition_kind.as_str())
        .bind(&draft.metadata)
        .fetch_one(&self.pool)
        .await?;
        map_competition_row(row)
    }
}
