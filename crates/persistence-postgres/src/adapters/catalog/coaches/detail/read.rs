use crate::{
    adapters::catalog::coaches::mapping::{
        coach_from_row, coach_name_from_row, external_id_from_row,
    },
    PersistenceError, PersistenceResult, PostgresStore,
};
use football_domain::CoachDetail;
use uuid::Uuid;
impl PostgresStore {
    pub async fn read_coach(&self, coach_id: Uuid) -> PersistenceResult<CoachDetail> {
        let row = sqlx::query(
            r#"
                SELECT id, canonical_name, normalized_name, nationality_code, status,
                       metadata, created_at, updated_at
                FROM football.coaches WHERE id=$1
                "#,
        )
        .bind(coach_id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| PersistenceError::InvalidState("教练不存在".to_string()))?;
        let coach = coach_from_row(&row)?;
        let names = sqlx::query(
            r#"
                SELECT id, coach_id, name, normalized_name, language_code, is_primary,
                       valid_from, valid_to
                FROM football.coach_names
                WHERE coach_id=$1
                ORDER BY is_primary DESC, valid_from DESC NULLS LAST, name, id
                "#,
        )
        .bind(coach_id)
        .fetch_all(&self.pool)
        .await?
        .iter()
        .map(coach_name_from_row)
        .collect::<PersistenceResult<Vec<_>>>()?;
        let team_periods = self.list_coach_team_periods(coach_id).await?;
        let external_ids = sqlx::query(
                r#"
                SELECT external.id, external.provider_id, provider.name AS provider_name, external.entity_type,
                       external.entity_id, external.external_id, external.metadata
                FROM football.external_entity_ids external
                JOIN catalog.data_providers provider ON provider.id=external.provider_id
                WHERE external.entity_type='coach' AND external.entity_id=$1
                ORDER BY provider.name, external.external_id
                "#,
            )
            .bind(coach_id)
            .fetch_all(&self.pool)
            .await?
            .iter()
            .map(external_id_from_row)
            .collect::<PersistenceResult<Vec<_>>>()?;
        Ok(CoachDetail {
            coach,
            names,
            team_periods,
            external_ids,
        })
    }
}
