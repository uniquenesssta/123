use super::super::port_registry::PersistenceStore;
use super::map_persistence_error;
use crate::ports::{competition::CompetitionHierarchyPort, PortResult};
use async_trait::async_trait;
use football_domain::{
    CompetitionDraft, CompetitionRecord, RoundDraft, RoundRecord, SeasonDraft, SeasonRecord,
    StageDraft, StageRecord,
};
use uuid::Uuid;

#[async_trait]
impl CompetitionHierarchyPort for PersistenceStore {
    async fn create_competition(&self, draft: &CompetitionDraft) -> PortResult<CompetitionRecord> {
        PersistenceStore::create_competition(self, draft)
            .await
            .map_err(map_persistence_error)
    }

    async fn delete_competition(&self, competition_id: Uuid) -> PortResult<()> {
        PersistenceStore::delete_competition(self, competition_id)
            .await
            .map_err(map_persistence_error)
    }

    async fn list_competitions(&self) -> PortResult<Vec<CompetitionRecord>> {
        PersistenceStore::list_competitions(self)
            .await
            .map_err(map_persistence_error)
    }

    async fn create_season(&self, draft: &SeasonDraft) -> PortResult<SeasonRecord> {
        PersistenceStore::create_season(self, draft)
            .await
            .map_err(map_persistence_error)
    }

    async fn list_seasons(&self) -> PortResult<Vec<SeasonRecord>> {
        PersistenceStore::list_seasons(self)
            .await
            .map_err(map_persistence_error)
    }

    async fn create_stage(&self, draft: &StageDraft) -> PortResult<StageRecord> {
        PersistenceStore::create_stage(self, draft)
            .await
            .map_err(map_persistence_error)
    }

    async fn list_stages(&self) -> PortResult<Vec<StageRecord>> {
        PersistenceStore::list_stages(self)
            .await
            .map_err(map_persistence_error)
    }

    async fn create_round(&self, draft: &RoundDraft) -> PortResult<RoundRecord> {
        PersistenceStore::create_round(self, draft)
            .await
            .map_err(map_persistence_error)
    }

    async fn list_rounds(&self) -> PortResult<Vec<RoundRecord>> {
        PersistenceStore::list_rounds(self)
            .await
            .map_err(map_persistence_error)
    }
}
