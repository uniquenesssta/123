use crate::{
    ports::player::{CoachCatalogPort, EntityReferencePort, PlayerCatalogPort, PlayerSignalPort},
    use_cases::players::{
        add_coach_name, add_external_entity_id, add_player_ability_observation,
        add_player_availability, add_player_dynamic_tag, add_player_name, add_player_team_period,
        add_team_coach_period, assign_player_position, bulk_archive_entities, bulk_delete_players,
        calculate_player_match_contribution, check_entity_deletion, create_coach,
        create_data_provider, create_player, delete_player, list_coaches, list_entity_references,
        list_players, read_coach, read_player, reference_data, resolve_entity_reference,
        update_player,
    },
    ApplicationResult,
};
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
pub(crate) struct PlayerService;
impl PlayerService {
    pub(crate) fn new() -> Self {
        Self
    }
    pub(crate) async fn reference_data<P: PlayerCatalogPort + ?Sized>(
        &self,
        port: &P,
    ) -> ApplicationResult<PlayerCatalogReferenceData> {
        reference_data::execute(port).await
    }
    pub(crate) async fn create_player<P: PlayerCatalogPort + ?Sized>(
        &self,
        port: &P,
        draft: PlayerDraft,
    ) -> ApplicationResult<PlayerRecord> {
        create_player::execute(port, draft).await
    }
    pub(crate) async fn update_player<P: PlayerCatalogPort + ?Sized>(
        &self,
        port: &P,
        player_id: Uuid,
        draft: PlayerDraft,
    ) -> ApplicationResult<PlayerRecord> {
        update_player::execute(port, player_id, draft).await
    }
    pub(crate) async fn delete_player<P: PlayerCatalogPort + ?Sized>(
        &self,
        port: &P,
        player_id: Uuid,
    ) -> ApplicationResult<()> {
        delete_player::execute(port, player_id).await
    }
    pub(crate) async fn bulk_delete_players<P: PlayerCatalogPort + ?Sized>(
        &self,
        port: &P,
        player_ids: Vec<Uuid>,
    ) -> ApplicationResult<BulkDeleteResult> {
        bulk_delete_players::execute(port, player_ids).await
    }
    pub(crate) async fn list_players<P: PlayerCatalogPort + ?Sized>(
        &self,
        port: &P,
        query: PlayerListQuery,
    ) -> ApplicationResult<PlayerListPage> {
        list_players::execute(port, query).await
    }
    pub(crate) async fn read_player<P: PlayerCatalogPort + ?Sized>(
        &self,
        port: &P,
        player_id: Uuid,
    ) -> ApplicationResult<PlayerDetail> {
        read_player::execute(port, player_id).await
    }
    pub(crate) async fn add_player_name<P: PlayerCatalogPort + ?Sized>(
        &self,
        port: &P,
        draft: PlayerNameDraft,
    ) -> ApplicationResult<PlayerNameRecord> {
        add_player_name::execute(port, draft).await
    }
    pub(crate) async fn assign_player_position<P: PlayerCatalogPort + ?Sized>(
        &self,
        port: &P,
        draft: PlayerPositionDraft,
    ) -> ApplicationResult<PlayerPositionRecord> {
        assign_player_position::execute(port, draft).await
    }
    pub(crate) async fn add_player_team_period<P: PlayerCatalogPort + ?Sized>(
        &self,
        port: &P,
        draft: PlayerTeamPeriodDraft,
    ) -> ApplicationResult<PlayerTeamPeriodRecord> {
        add_player_team_period::execute(port, draft).await
    }
    pub(crate) async fn add_player_availability<P: PlayerSignalPort + ?Sized>(
        &self,
        port: &P,
        draft: PlayerAvailabilityDraft,
    ) -> ApplicationResult<PlayerAvailabilityRecord> {
        add_player_availability::execute(port, draft).await
    }
    pub(crate) async fn add_player_ability_observation<P: PlayerSignalPort + ?Sized>(
        &self,
        port: &P,
        draft: PlayerAbilityObservationDraft,
    ) -> ApplicationResult<PlayerAbilityObservationRecord> {
        add_player_ability_observation::execute(port, draft).await
    }
    pub(crate) async fn add_player_dynamic_tag<P: PlayerSignalPort + ?Sized>(
        &self,
        port: &P,
        draft: PlayerDynamicTagDraft,
    ) -> ApplicationResult<PlayerDynamicTagRecord> {
        add_player_dynamic_tag::execute(port, draft).await
    }
    pub(crate) async fn calculate_player_match_contribution<P: PlayerSignalPort + ?Sized>(
        &self,
        port: &P,
        request: PlayerMatchContributionRequest,
    ) -> ApplicationResult<PlayerMatchContribution> {
        calculate_player_match_contribution::execute(port, request).await
    }
    pub(crate) async fn create_coach<P: CoachCatalogPort + ?Sized>(
        &self,
        port: &P,
        draft: CoachDraft,
    ) -> ApplicationResult<CoachRecord> {
        create_coach::execute(port, draft).await
    }
    pub(crate) async fn list_coaches<P: CoachCatalogPort + ?Sized>(
        &self,
        port: &P,
        query: CoachListQuery,
    ) -> ApplicationResult<Vec<CoachListItem>> {
        list_coaches::execute(port, query).await
    }
    pub(crate) async fn read_coach<P: CoachCatalogPort + ?Sized>(
        &self,
        port: &P,
        coach_id: Uuid,
    ) -> ApplicationResult<CoachDetail> {
        read_coach::execute(port, coach_id).await
    }
    pub(crate) async fn add_coach_name<P: CoachCatalogPort + ?Sized>(
        &self,
        port: &P,
        draft: CoachNameDraft,
    ) -> ApplicationResult<CoachNameRecord> {
        add_coach_name::execute(port, draft).await
    }
    pub(crate) async fn add_team_coach_period<P: CoachCatalogPort + ?Sized>(
        &self,
        port: &P,
        draft: TeamCoachPeriodDraft,
    ) -> ApplicationResult<TeamCoachPeriodRecord> {
        add_team_coach_period::execute(port, draft).await
    }
    pub(crate) async fn list_entity_references<P: EntityReferencePort + ?Sized>(
        &self,
        port: &P,
        query: EntityReferenceQuery,
    ) -> ApplicationResult<Vec<EntityReferenceRecord>> {
        list_entity_references::execute(port, query).await
    }
    pub(crate) async fn resolve_entity_reference<P: EntityReferencePort + ?Sized>(
        &self,
        port: &P,
        request: EntityMatchRequest,
    ) -> ApplicationResult<EntityMatchResult> {
        resolve_entity_reference::execute(port, request).await
    }
    pub(crate) async fn check_entity_deletion<P: EntityReferencePort + ?Sized>(
        &self,
        port: &P,
        entity_type: String,
        entity_id: Uuid,
    ) -> ApplicationResult<EntityDeletionCheck> {
        check_entity_deletion::execute(port, entity_type, entity_id).await
    }
    pub(crate) async fn bulk_archive_entities<P: EntityReferencePort + ?Sized>(
        &self,
        port: &P,
        entity_type: String,
        entity_ids: Vec<Uuid>,
    ) -> ApplicationResult<BulkArchiveResult> {
        bulk_archive_entities::execute(port, entity_type, entity_ids).await
    }
    pub(crate) async fn create_data_provider<P: EntityReferencePort + ?Sized>(
        &self,
        port: &P,
        draft: DataProviderDraft,
    ) -> ApplicationResult<DataProviderRecord> {
        create_data_provider::execute(port, draft).await
    }
    pub(crate) async fn add_external_entity_id<P: EntityReferencePort + ?Sized>(
        &self,
        port: &P,
        draft: ExternalEntityIdDraft,
    ) -> ApplicationResult<ExternalEntityIdRecord> {
        add_external_entity_id::execute(port, draft).await
    }
}
