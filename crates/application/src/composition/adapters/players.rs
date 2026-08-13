use super::super::port_registry::PersistenceStore;
use super::map_persistence_error;
use crate::ports::{
    player::{CoachCatalogPort, EntityReferencePort, PlayerCatalogPort, PlayerSignalPort},
    PortResult,
};
use async_trait::async_trait;
use football_domain::{
    BulkArchiveResult, BulkDeleteResult, CoachDetail, CoachDraft, CoachListItem, CoachListQuery,
    CoachNameDraft, CoachNameRecord, CoachRecord, DataProviderDraft, DataProviderRecord,
    EntityDeletionCheck, EntityMatchRequest, EntityMatchResult, EntityReferenceQuery,
    EntityReferenceRecord, ExternalEntityIdDraft, ExternalEntityIdRecord,
    PlayerAbilityObservationDraft, PlayerAbilityObservationRecord, PlayerAvailabilityDraft,
    PlayerAvailabilityRecord, PlayerCatalogReferenceData, PlayerDetail, PlayerDraft,
    PlayerDynamicTagDraft, PlayerDynamicTagRecord, PlayerListPage, PlayerListQuery,
    PlayerMatchContribution, PlayerMatchContributionRequest, PlayerNameDraft, PlayerNameRecord,
    PlayerPositionDraft, PlayerPositionRecord, PlayerRecord, PlayerTeamPeriodDraft,
    PlayerTeamPeriodRecord, TeamCoachPeriodDraft, TeamCoachPeriodRecord,
};
use uuid::Uuid;

#[async_trait]
impl PlayerCatalogPort for PersistenceStore {
    async fn reference_data(&self) -> PortResult<PlayerCatalogReferenceData> {
        self.player_catalog_reference_data()
            .await
            .map_err(map_persistence_error)
    }
    async fn create_player(&self, draft: &PlayerDraft) -> PortResult<PlayerRecord> {
        self.create_player(draft)
            .await
            .map_err(map_persistence_error)
    }
    async fn update_player(
        &self,
        player_id: Uuid,
        draft: &PlayerDraft,
    ) -> PortResult<PlayerRecord> {
        self.update_player(player_id, draft)
            .await
            .map_err(map_persistence_error)
    }
    async fn delete_player(&self, player_id: Uuid) -> PortResult<()> {
        self.delete_player(player_id)
            .await
            .map_err(map_persistence_error)
    }
    async fn bulk_delete_players(&self, player_ids: &[Uuid]) -> PortResult<BulkDeleteResult> {
        self.bulk_delete_players(player_ids)
            .await
            .map_err(map_persistence_error)
    }
    async fn list_players(&self, query: &PlayerListQuery) -> PortResult<PlayerListPage> {
        self.list_players(query)
            .await
            .map_err(map_persistence_error)
    }
    async fn read_player(&self, player_id: Uuid) -> PortResult<PlayerDetail> {
        self.read_player(player_id)
            .await
            .map_err(map_persistence_error)
    }
    async fn add_player_name(&self, draft: &PlayerNameDraft) -> PortResult<PlayerNameRecord> {
        self.add_player_name(draft)
            .await
            .map_err(map_persistence_error)
    }
    async fn assign_player_position(
        &self,
        draft: &PlayerPositionDraft,
    ) -> PortResult<PlayerPositionRecord> {
        self.assign_player_position(draft)
            .await
            .map_err(map_persistence_error)
    }
    async fn add_player_team_period(
        &self,
        draft: &PlayerTeamPeriodDraft,
    ) -> PortResult<PlayerTeamPeriodRecord> {
        self.add_player_team_period(draft)
            .await
            .map_err(map_persistence_error)
    }
}

#[async_trait]
impl PlayerSignalPort for PersistenceStore {
    async fn add_availability(
        &self,
        draft: &PlayerAvailabilityDraft,
    ) -> PortResult<PlayerAvailabilityRecord> {
        self.add_player_availability(draft)
            .await
            .map_err(map_persistence_error)
    }
    async fn add_ability_observation(
        &self,
        draft: &PlayerAbilityObservationDraft,
    ) -> PortResult<PlayerAbilityObservationRecord> {
        self.add_player_ability_observation(draft)
            .await
            .map_err(map_persistence_error)
    }
    async fn add_dynamic_tag(
        &self,
        draft: &PlayerDynamicTagDraft,
    ) -> PortResult<PlayerDynamicTagRecord> {
        self.add_player_dynamic_tag(draft)
            .await
            .map_err(map_persistence_error)
    }
    async fn calculate_match_contribution(
        &self,
        request: &PlayerMatchContributionRequest,
    ) -> PortResult<PlayerMatchContribution> {
        self.calculate_player_match_contribution(request)
            .await
            .map_err(map_persistence_error)
    }
}

#[async_trait]
impl CoachCatalogPort for PersistenceStore {
    async fn create_coach(&self, draft: &CoachDraft) -> PortResult<CoachRecord> {
        self.create_coach(draft)
            .await
            .map_err(map_persistence_error)
    }
    async fn list_coaches(&self, query: &CoachListQuery) -> PortResult<Vec<CoachListItem>> {
        self.list_coaches(query)
            .await
            .map_err(map_persistence_error)
    }
    async fn read_coach(&self, coach_id: Uuid) -> PortResult<CoachDetail> {
        self.read_coach(coach_id)
            .await
            .map_err(map_persistence_error)
    }
    async fn add_coach_name(&self, draft: &CoachNameDraft) -> PortResult<CoachNameRecord> {
        self.add_coach_name(draft)
            .await
            .map_err(map_persistence_error)
    }
    async fn add_team_coach_period(
        &self,
        draft: &TeamCoachPeriodDraft,
    ) -> PortResult<TeamCoachPeriodRecord> {
        self.add_team_coach_period(draft)
            .await
            .map_err(map_persistence_error)
    }
}

#[async_trait]
impl EntityReferencePort for PersistenceStore {
    async fn list_references(
        &self,
        query: &EntityReferenceQuery,
    ) -> PortResult<Vec<EntityReferenceRecord>> {
        self.list_entity_references(query)
            .await
            .map_err(map_persistence_error)
    }
    async fn resolve_reference(
        &self,
        request: &EntityMatchRequest,
    ) -> PortResult<EntityMatchResult> {
        self.resolve_entity_reference(request)
            .await
            .map_err(map_persistence_error)
    }
    async fn check_deletion(
        &self,
        entity_type: &str,
        entity_id: Uuid,
    ) -> PortResult<EntityDeletionCheck> {
        self.check_entity_deletion(entity_type, entity_id)
            .await
            .map_err(map_persistence_error)
    }
    async fn bulk_archive(
        &self,
        entity_type: &str,
        entity_ids: &[Uuid],
    ) -> PortResult<BulkArchiveResult> {
        self.bulk_archive_entities(entity_type, entity_ids)
            .await
            .map_err(map_persistence_error)
    }
    async fn create_data_provider(
        &self,
        draft: &DataProviderDraft,
    ) -> PortResult<DataProviderRecord> {
        self.create_data_provider(draft)
            .await
            .map_err(map_persistence_error)
    }
    async fn add_external_id(
        &self,
        draft: &ExternalEntityIdDraft,
    ) -> PortResult<ExternalEntityIdRecord> {
        self.add_external_entity_id(draft)
            .await
            .map_err(map_persistence_error)
    }
}
