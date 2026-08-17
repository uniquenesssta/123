use super::{
    input_policy::validate_player_ability_observation, mapper::map_player_ability_observation,
    row::PlayerAbilityObservationRow,
};
use crate::{PersistenceError, PersistenceResult, PostgresStore};
use football_domain::{PlayerAbilityObservationDraft, PlayerAbilityObservationRecord};
use uuid::Uuid;

impl PostgresStore {
    pub async fn add_player_ability_observation(
        &self,
        draft: &PlayerAbilityObservationDraft,
    ) -> PersistenceResult<PlayerAbilityObservationRecord> {
        validate_player_ability_observation(draft)?;
        let row = sqlx::query_as::<_, PlayerAbilityObservationRow>(
            r#"
            WITH dimension AS (
                SELECT code, name, minimum_value, maximum_value
                FROM feature.player_ability_dimensions
                WHERE code = $2
            ), inserted AS (
                INSERT INTO feature.player_ability_observations (
                    id, player_id, dimension_code, context_type, context_id,
                    value, confidence, sample_size, observed_at,
                    effective_from, effective_to, calculation_version,
                    source_document_id, metadata
                )
                SELECT
                    $1, $3, dimension.code, $4, $5, $6, $7, $8, $9,
                    $10, $11, $12, $13, $14
                FROM dimension
                WHERE $6 BETWEEN dimension.minimum_value AND dimension.maximum_value
                RETURNING *
            )
            SELECT
                inserted.id, inserted.player_id, inserted.dimension_code,
                dimension.name AS dimension_name, inserted.context_type,
                inserted.context_id, inserted.value, inserted.confidence,
                inserted.sample_size, inserted.observed_at,
                inserted.effective_from, inserted.effective_to,
                inserted.calculation_version
            FROM inserted
            JOIN dimension ON dimension.code = inserted.dimension_code
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(draft.dimension_code.trim())
        .bind(draft.player_id)
        .bind(draft.context_type.trim())
        .bind(draft.context_id)
        .bind(draft.value)
        .bind(draft.confidence)
        .bind(draft.sample_size)
        .bind(draft.observed_at)
        .bind(draft.effective_from)
        .bind(draft.effective_to)
        .bind(draft.calculation_version.trim())
        .bind(draft.source_document_id)
        .bind(&draft.metadata)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| {
            PersistenceError::InvalidState("能力维度不存在，或能力值超出该维度允许范围".to_string())
        })?;
        Ok(map_player_ability_observation(row))
    }
}
