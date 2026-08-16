use super::{
    input_policy::validate_player_draft, record_mapper::map_player_record,
    record_row::PlayerRecordRow,
};
use crate::{
    adapters::catalog::players::normalization::normalize_name, PersistenceError, PersistenceResult,
    PostgresStore,
};
use football_domain::{PlayerDraft, PlayerRecord};
use serde_json::json;
use uuid::Uuid;

impl PostgresStore {
    pub async fn update_player(
        &self,
        player_id: Uuid,
        draft: &PlayerDraft,
    ) -> PersistenceResult<PlayerRecord> {
        let validated = validate_player_draft(draft)?;
        let normalized_name = normalize_name(validated.canonical_name);
        let mut tx = self.pool.begin().await?;
        let previous_normalized_name: String = sqlx::query_scalar(
            "SELECT normalized_name FROM football.players WHERE id = $1 FOR UPDATE",
        )
        .bind(player_id)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| PersistenceError::InvalidState("球员不存在".to_string()))?;
        let row = sqlx::query_as::<_, PlayerRecordRow>(
            r#"
            UPDATE football.players SET
                canonical_name = $2,
                normalized_name = $3,
                date_of_birth = $4,
                nationality_code = $5,
                preferred_foot = $6,
                height_cm = $7,
                status = $8,
                metadata = metadata || $9,
                updated_at = now()
            WHERE id = $1
            RETURNING id, canonical_name, normalized_name, date_of_birth,
                      nationality_code, preferred_foot, height_cm, status, created_at
            "#,
        )
        .bind(player_id)
        .bind(validated.canonical_name)
        .bind(&normalized_name)
        .bind(draft.date_of_birth)
        .bind(validated.nationality_code)
        .bind(draft.preferred_foot.as_str())
        .bind(draft.height_cm)
        .bind(draft.status.as_str())
        .bind(&draft.metadata)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| PersistenceError::InvalidState("球员不存在".to_string()))?;
        if previous_normalized_name != normalized_name {
            sqlx::query("UPDATE football.player_names SET is_primary = false WHERE player_id = $1")
                .bind(player_id)
                .execute(&mut *tx)
                .await?;
            sqlx::query(
                r#"
                INSERT INTO football.player_names (
                    id, player_id, name, normalized_name, is_primary
                ) VALUES ($1, $2, $3, $4, true)
                "#,
            )
            .bind(Uuid::new_v4())
            .bind(player_id)
            .bind(validated.canonical_name)
            .bind(&normalized_name)
            .execute(&mut *tx)
            .await?;
        }
        crate::write_audit_event(
            &mut tx,
            "player_updated",
            "player",
            player_id.to_string(),
            json!({"canonical_name": validated.canonical_name, "source": "manual"}),
        )
        .await?;
        tx.commit().await?;
        map_player_record(row)
    }
}
