use super::input_policy::validate_player_draft;
use crate::{
    adapters::catalog::players::{
        normalization::normalize_name,
        record::{map_player_record, PlayerRecordRow},
    },
    PersistenceResult, PostgresStore,
};
use football_domain::{PlayerDraft, PlayerRecord};
use serde_json::json;
use uuid::Uuid;

impl PostgresStore {
    pub async fn create_player(&self, draft: &PlayerDraft) -> PersistenceResult<PlayerRecord> {
        let validated = validate_player_draft(draft)?;
        let normalized_name = normalize_name(validated.canonical_name);
        let id = Uuid::new_v4();
        let mut tx = self.pool.begin().await?;
        let row = sqlx::query_as::<_, PlayerRecordRow>(
            r#"
            INSERT INTO football.players (
                id, canonical_name, normalized_name, date_of_birth,
                nationality_code, preferred_foot, height_cm, status, metadata
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            RETURNING
                id, canonical_name, normalized_name, date_of_birth,
                nationality_code, preferred_foot, height_cm, status, created_at
            "#,
        )
        .bind(id)
        .bind(validated.canonical_name)
        .bind(&normalized_name)
        .bind(draft.date_of_birth)
        .bind(validated.nationality_code)
        .bind(draft.preferred_foot.as_str())
        .bind(draft.height_cm)
        .bind(draft.status.as_str())
        .bind(&draft.metadata)
        .fetch_one(&mut *tx)
        .await?;

        sqlx::query(
            r#"
            INSERT INTO football.player_names (
                id, player_id, name, normalized_name, is_primary
            ) VALUES ($1, $2, $3, $4, true)
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(id)
        .bind(validated.canonical_name)
        .bind(&normalized_name)
        .execute(&mut *tx)
        .await?;
        crate::write_audit_event(
            &mut tx,
            "player_created",
            "player",
            id.to_string(),
            json!({"canonical_name": validated.canonical_name}),
        )
        .await?;
        tx.commit().await?;
        map_player_record(row)
    }
}
