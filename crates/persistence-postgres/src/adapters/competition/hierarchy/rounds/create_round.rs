use super::read_round;
use crate::{PersistenceResult, PostgresStore};
use football_domain::{RoundDraft, RoundRecord};
use uuid::Uuid;

impl PostgresStore {
    pub async fn create_round(&self, draft: &RoundDraft) -> PersistenceResult<RoundRecord> {
        let id = Uuid::new_v4();
        sqlx::query(
            r#"
            INSERT INTO football.rounds (
                id, stage_id, code, name, sequence_no, starts_at, ends_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7)
            "#,
        )
        .bind(id)
        .bind(draft.stage_id)
        .bind(draft.code.trim())
        .bind(draft.name.trim())
        .bind(draft.sequence_no)
        .bind(draft.starts_at)
        .bind(draft.ends_at)
        .execute(&self.pool)
        .await?;
        read_round(self, id).await
    }
}
