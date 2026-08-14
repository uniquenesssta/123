use super::read_stage;
use crate::{PersistenceResult, PostgresStore};
use football_domain::{StageDraft, StageRecord};
use uuid::Uuid;

impl PostgresStore {
    pub async fn create_stage(&self, draft: &StageDraft) -> PersistenceResult<StageRecord> {
        let id = Uuid::new_v4();
        sqlx::query(
            r#"
            INSERT INTO football.competition_stages (
                id, season_id, code, name, stage_kind, sequence_no, rules
            ) VALUES ($1, $2, $3, $4, $5, $6, $7)
            "#,
        )
        .bind(id)
        .bind(draft.season_id)
        .bind(draft.code.trim())
        .bind(draft.name.trim())
        .bind(draft.stage_kind.as_str())
        .bind(draft.sequence_no)
        .bind(&draft.rules)
        .execute(&self.pool)
        .await?;
        read_stage(self, id).await
    }
}
