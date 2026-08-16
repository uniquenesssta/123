use super::{input_policy::validate_player_name_draft, map_player_name, row::PlayerNameRow};
use crate::{
    adapters::catalog::players::normalization::normalize_name, PersistenceResult, PostgresStore,
};
use football_domain::{PlayerNameDraft, PlayerNameRecord};
use uuid::Uuid;

impl PostgresStore {
    pub async fn add_player_name(
        &self,
        draft: &PlayerNameDraft,
    ) -> PersistenceResult<PlayerNameRecord> {
        let validated = validate_player_name_draft(draft)?;
        let normalized_name = normalize_name(validated.name);
        let mut tx = self.pool.begin().await?;
        if draft.is_primary {
            sqlx::query("UPDATE football.player_names SET is_primary = false WHERE player_id = $1")
                .bind(draft.player_id)
                .execute(&mut *tx)
                .await?;
            sqlx::query(
                r#"
      UPDATE football.players
      SET canonical_name = $2, normalized_name = $3, updated_at = now()
      WHERE id = $1
      "#,
            )
            .bind(draft.player_id)
            .bind(validated.name)
            .bind(&normalized_name)
            .execute(&mut *tx)
            .await?;
        }
        let row = sqlx::query_as::<_, PlayerNameRow>(
            r#"
  INSERT INTO football.player_names (
      id, player_id, name, normalized_name, language_code,
      is_primary, valid_from, valid_to
  ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
  RETURNING id, player_id, name, normalized_name, language_code,
            is_primary, valid_from, valid_to
  "#,
        )
        .bind(Uuid::new_v4())
        .bind(draft.player_id)
        .bind(validated.name)
        .bind(&normalized_name)
        .bind(validated.language_code)
        .bind(draft.is_primary)
        .bind(draft.valid_from)
        .bind(draft.valid_to)
        .fetch_one(&mut *tx)
        .await?;
        tx.commit().await?;
        Ok(map_player_name(row))
    }
}
