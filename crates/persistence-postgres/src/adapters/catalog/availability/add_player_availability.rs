use super::{
    input_policy::validate_player_availability_draft, mapper::map_player_availability,
    row::PlayerAvailabilityRow,
};
use crate::{PersistenceResult, PostgresStore};
use football_domain::{PlayerAvailabilityDraft, PlayerAvailabilityRecord};
use uuid::Uuid;

impl PostgresStore {
    pub async fn add_player_availability(
        &self,
        draft: &PlayerAvailabilityDraft,
    ) -> PersistenceResult<PlayerAvailabilityRecord> {
        let validated = validate_player_availability_draft(draft)?;
        let row = sqlx::query_as::<_, PlayerAvailabilityRow>(
            r#"
            WITH inserted AS (
                INSERT INTO football.player_availability (
                    id, player_id, team_id, competition_id, status, reason,
                    confidence, valid_from, valid_to, source_document_id, metadata
                ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
                RETURNING *
            )
            SELECT
                inserted.id, inserted.player_id, inserted.team_id,
                team.canonical_name AS team_name, inserted.competition_id,
                inserted.status, inserted.reason, inserted.confidence,
                inserted.valid_from, inserted.valid_to, inserted.created_at
            FROM inserted
            LEFT JOIN football.teams team ON team.id = inserted.team_id
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(draft.player_id)
        .bind(draft.team_id)
        .bind(draft.competition_id)
        .bind(draft.status.as_str())
        .bind(validated.reason)
        .bind(draft.confidence)
        .bind(draft.valid_from)
        .bind(draft.valid_to)
        .bind(draft.source_document_id)
        .bind(&draft.metadata)
        .fetch_one(&self.pool)
        .await?;
        map_player_availability(row)
    }
}
