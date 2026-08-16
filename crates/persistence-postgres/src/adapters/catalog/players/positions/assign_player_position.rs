use super::{
    input_policy::validate_player_position_draft, map_player_position, row::PlayerPositionRow,
};
use crate::{PersistenceResult, PostgresStore};
use football_domain::{PlayerPositionDraft, PlayerPositionRecord};
use uuid::Uuid;

impl PostgresStore {
    pub async fn assign_player_position(
        &self,
        draft: &PlayerPositionDraft,
    ) -> PersistenceResult<PlayerPositionRecord> {
        let validated = validate_player_position_draft(draft)?;
        let mut tx = self.pool.begin().await?;
        if draft.is_primary {
            sqlx::query(
                "UPDATE football.player_positions SET is_primary = false WHERE player_id = $1",
            )
            .bind(draft.player_id)
            .execute(&mut *tx)
            .await?;
        }
        let row = sqlx::query_as::<_, PlayerPositionRow>(
            r#"
  WITH inserted AS (
      INSERT INTO football.player_positions (
          id, player_id, position_code, proficiency, default_role_code, is_primary,
          valid_from, valid_to, source_document_id
      ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
      RETURNING *
  )
  SELECT
      inserted.id, inserted.player_id, inserted.position_code,
      position.name AS position_name, position.position_group,
      inserted.proficiency, inserted.default_role_code, inserted.is_primary,
      inserted.valid_from, inserted.valid_to
  FROM inserted
  JOIN football.positions position ON position.code = inserted.position_code
  "#,
        )
        .bind(Uuid::new_v4())
        .bind(draft.player_id)
        .bind(&validated.position_code)
        .bind(draft.proficiency)
        .bind(validated.default_role_code)
        .bind(draft.is_primary)
        .bind(draft.valid_from)
        .bind(draft.valid_to)
        .bind(draft.source_document_id)
        .fetch_one(&mut *tx)
        .await?;
        tx.commit().await?;
        Ok(map_player_position(row))
    }
}
