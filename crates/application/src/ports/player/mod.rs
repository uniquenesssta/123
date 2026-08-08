use crate::ports::PortResult;
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
pub trait PlayerCatalogPort: Send + Sync {
    async fn reference_data(&self) -> PortResult<PlayerCatalogReferenceData>;
    async fn create_player(&self, draft: &PlayerDraft) -> PortResult<PlayerRecord>;
    async fn update_player(&self, player_id: Uuid, draft: &PlayerDraft)
        -> PortResult<PlayerRecord>;
    async fn delete_player(&self, player_id: Uuid) -> PortResult<()>;
    async fn bulk_delete_players(&self, player_ids: &[Uuid]) -> PortResult<BulkDeleteResult>;
    async fn list_players(&self, query: &PlayerListQuery) -> PortResult<PlayerListPage>;
    async fn read_player(&self, player_id: Uuid) -> PortResult<PlayerDetail>;
    async fn add_player_name(&self, draft: &PlayerNameDraft) -> PortResult<PlayerNameRecord>;
    async fn assign_player_position(
        &self,
        draft: &PlayerPositionDraft,
    ) -> PortResult<PlayerPositionRecord>;
    async fn add_player_team_period(
        &self,
        draft: &PlayerTeamPeriodDraft,
    ) -> PortResult<PlayerTeamPeriodRecord>;
}

#[async_trait]
pub trait PlayerSignalPort: Send + Sync {
    async fn add_availability(
        &self,
        draft: &PlayerAvailabilityDraft,
    ) -> PortResult<PlayerAvailabilityRecord>;
    async fn add_ability_observation(
        &self,
        draft: &PlayerAbilityObservationDraft,
    ) -> PortResult<PlayerAbilityObservationRecord>;
    async fn add_dynamic_tag(
        &self,
        draft: &PlayerDynamicTagDraft,
    ) -> PortResult<PlayerDynamicTagRecord>;
    async fn calculate_match_contribution(
        &self,
        request: &PlayerMatchContributionRequest,
    ) -> PortResult<PlayerMatchContribution>;
}

#[async_trait]
pub trait CoachCatalogPort: Send + Sync {
    async fn create_coach(&self, draft: &CoachDraft) -> PortResult<CoachRecord>;
    async fn list_coaches(&self, query: &CoachListQuery) -> PortResult<Vec<CoachListItem>>;
    async fn read_coach(&self, coach_id: Uuid) -> PortResult<CoachDetail>;
    async fn add_coach_name(&self, draft: &CoachNameDraft) -> PortResult<CoachNameRecord>;
    async fn add_team_coach_period(
        &self,
        draft: &TeamCoachPeriodDraft,
    ) -> PortResult<TeamCoachPeriodRecord>;
}

#[async_trait]
pub trait EntityReferencePort: Send + Sync {
    async fn list_references(
        &self,
        query: &EntityReferenceQuery,
    ) -> PortResult<Vec<EntityReferenceRecord>>;
    async fn resolve_reference(
        &self,
        request: &EntityMatchRequest,
    ) -> PortResult<EntityMatchResult>;
    async fn check_deletion(
        &self,
        entity_type: &str,
        entity_id: Uuid,
    ) -> PortResult<EntityDeletionCheck>;
    async fn bulk_archive(
        &self,
        entity_type: &str,
        entity_ids: &[Uuid],
    ) -> PortResult<BulkArchiveResult>;
    async fn create_data_provider(
        &self,
        draft: &DataProviderDraft,
    ) -> PortResult<DataProviderRecord>;
    async fn add_external_id(
        &self,
        draft: &ExternalEntityIdDraft,
    ) -> PortResult<ExternalEntityIdRecord>;
}
