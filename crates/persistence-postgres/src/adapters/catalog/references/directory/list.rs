use super::read::{coach_references, player_references, team_references};
use crate::{
    adapters::catalog::references::validate_entity_type, PersistenceResult, PostgresStore,
};
use football_domain::{EntityReferenceQuery, EntityReferenceRecord};

impl PostgresStore {
    pub async fn list_entity_references(
        &self,
        query: &EntityReferenceQuery,
    ) -> PersistenceResult<Vec<EntityReferenceRecord>> {
        validate_entity_type(&query.entity_type)?;
        match query.entity_type.as_str() {
            "team" => team_references(&self.pool, query).await,
            "player" => player_references(&self.pool, query).await,
            "coach" => coach_references(&self.pool, query).await,
            _ => unreachable!(),
        }
    }
}
