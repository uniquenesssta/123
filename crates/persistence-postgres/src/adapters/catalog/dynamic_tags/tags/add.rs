use super::input_policy::validate_dynamic_tag_draft;
use crate::{PersistenceResult, PostgresStore};
use football_domain::{PlayerDynamicTagDraft, PlayerDynamicTagRecord};
use sqlx::Row;
use uuid::Uuid;

impl PostgresStore {
    pub async fn add_player_dynamic_tag(
        &self,
        draft: &PlayerDynamicTagDraft,
    ) -> PersistenceResult<PlayerDynamicTagRecord> {
        validate_dynamic_tag_draft(self, draft).await?;
        let id = Uuid::new_v4();
        let row = sqlx::query(
            r#"
            INSERT INTO feature.player_dynamic_tags (
                id, player_id, tag_code, value, label, confidence,
                observed_at, valid_from, valid_to, competition_id,
                position_code, opponent_team_id, sample_size, source_type,
                source_document_id, calculation_version, metadata
            ) VALUES (
                $1, $2, $3, $4, $5, $6,
                $7, $8, $9, $10,
                $11, $12, $13, $14,
                $15, $16, $17
            )
            RETURNING id
            "#,
        )
        .bind(id)
        .bind(draft.player_id)
        .bind(draft.tag_code.trim())
        .bind(draft.value)
        .bind(draft.label.as_deref())
        .bind(draft.confidence)
        .bind(draft.observed_at)
        .bind(draft.valid_from)
        .bind(draft.valid_to)
        .bind(draft.competition_id)
        .bind(draft.position_code.as_deref())
        .bind(draft.opponent_team_id)
        .bind(draft.sample_size)
        .bind(draft.source_type.trim())
        .bind(draft.source_document_id)
        .bind(draft.calculation_version.trim())
        .bind(&draft.metadata)
        .fetch_one(&self.pool)
        .await?;
        let inserted_id: Uuid = row.try_get("id")?;
        self.read_player_dynamic_tag(inserted_id).await
    }
}
