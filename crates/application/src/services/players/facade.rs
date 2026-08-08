use crate::{ApplicationError, ApplicationResult, ApplicationService};
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
impl ApplicationService {
    async fn player_session(&self) -> ApplicationResult<crate::composition::ActiveDatabase> {
        self.database
            .active_session()
            .await
            .ok_or(ApplicationError::DatabaseNotConnected)
    }
    pub async fn player_catalog_reference_data(
        &self,
    ) -> ApplicationResult<PlayerCatalogReferenceData> {
        let session = self.player_session().await?;
        self.players.reference_data(&session).await
    }
    pub async fn create_coach(&self, draft: CoachDraft) -> ApplicationResult<CoachRecord> {
        let session = self.player_session().await?;
        self.players.create_coach(&session, draft).await
    }
    pub async fn list_coaches(
        &self,
        query: CoachListQuery,
    ) -> ApplicationResult<Vec<CoachListItem>> {
        let session = self.player_session().await?;
        self.players.list_coaches(&session, query).await
    }
    pub async fn read_coach(&self, coach_id: Uuid) -> ApplicationResult<CoachDetail> {
        let session = self.player_session().await?;
        self.players.read_coach(&session, coach_id).await
    }
    pub async fn add_coach_name(
        &self,
        draft: CoachNameDraft,
    ) -> ApplicationResult<CoachNameRecord> {
        let session = self.player_session().await?;
        self.players.add_coach_name(&session, draft).await
    }
    pub async fn add_team_coach_period(
        &self,
        draft: TeamCoachPeriodDraft,
    ) -> ApplicationResult<TeamCoachPeriodRecord> {
        let session = self.player_session().await?;
        self.players.add_team_coach_period(&session, draft).await
    }
    pub async fn list_entity_references(
        &self,
        query: EntityReferenceQuery,
    ) -> ApplicationResult<Vec<EntityReferenceRecord>> {
        let session = self.player_session().await?;
        self.players.list_entity_references(&session, query).await
    }
    pub async fn resolve_entity_reference(
        &self,
        request: EntityMatchRequest,
    ) -> ApplicationResult<EntityMatchResult> {
        let session = self.player_session().await?;
        self.players
            .resolve_entity_reference(&session, request)
            .await
    }
    pub async fn check_entity_deletion(
        &self,
        entity_type: String,
        entity_id: Uuid,
    ) -> ApplicationResult<EntityDeletionCheck> {
        let session = self.player_session().await?;
        self.players
            .check_entity_deletion(&session, entity_type, entity_id)
            .await
    }
    pub async fn bulk_archive_entities(
        &self,
        entity_type: String,
        entity_ids: Vec<Uuid>,
    ) -> ApplicationResult<BulkArchiveResult> {
        let session = self.player_session().await?;
        self.players
            .bulk_archive_entities(&session, entity_type, entity_ids)
            .await
    }
    pub async fn create_data_provider(
        &self,
        draft: DataProviderDraft,
    ) -> ApplicationResult<DataProviderRecord> {
        let session = self.player_session().await?;
        self.players.create_data_provider(&session, draft).await
    }
    pub async fn create_player(&self, draft: PlayerDraft) -> ApplicationResult<PlayerRecord> {
        let session = self.player_session().await?;
        self.players.create_player(&session, draft).await
    }
    pub async fn update_player(
        &self,
        player_id: Uuid,
        draft: PlayerDraft,
    ) -> ApplicationResult<PlayerRecord> {
        let session = self.player_session().await?;
        self.players.update_player(&session, player_id, draft).await
    }
    pub async fn delete_player(&self, player_id: Uuid) -> ApplicationResult<()> {
        let session = self.player_session().await?;
        self.players.delete_player(&session, player_id).await
    }
    pub async fn bulk_delete_players(
        &self,
        player_ids: Vec<Uuid>,
    ) -> ApplicationResult<BulkDeleteResult> {
        let session = self.player_session().await?;
        self.players.bulk_delete_players(&session, player_ids).await
    }
    pub async fn list_players(&self, query: PlayerListQuery) -> ApplicationResult<PlayerListPage> {
        let session = self.player_session().await?;
        self.players.list_players(&session, query).await
    }
    pub async fn read_player(&self, player_id: Uuid) -> ApplicationResult<PlayerDetail> {
        let session = self.player_session().await?;
        self.players.read_player(&session, player_id).await
    }
    pub async fn add_player_name(
        &self,
        draft: PlayerNameDraft,
    ) -> ApplicationResult<PlayerNameRecord> {
        let session = self.player_session().await?;
        self.players.add_player_name(&session, draft).await
    }
    pub async fn assign_player_position(
        &self,
        draft: PlayerPositionDraft,
    ) -> ApplicationResult<PlayerPositionRecord> {
        let session = self.player_session().await?;
        self.players.assign_player_position(&session, draft).await
    }
    pub async fn add_player_team_period(
        &self,
        draft: PlayerTeamPeriodDraft,
    ) -> ApplicationResult<PlayerTeamPeriodRecord> {
        let session = self.player_session().await?;
        self.players.add_player_team_period(&session, draft).await
    }
    pub async fn add_player_availability(
        &self,
        draft: PlayerAvailabilityDraft,
    ) -> ApplicationResult<PlayerAvailabilityRecord> {
        let session = self.player_session().await?;
        self.players.add_player_availability(&session, draft).await
    }
    pub async fn add_player_ability_observation(
        &self,
        draft: PlayerAbilityObservationDraft,
    ) -> ApplicationResult<PlayerAbilityObservationRecord> {
        let session = self.player_session().await?;
        self.players
            .add_player_ability_observation(&session, draft)
            .await
    }
    pub async fn add_player_dynamic_tag(
        &self,
        draft: PlayerDynamicTagDraft,
    ) -> ApplicationResult<PlayerDynamicTagRecord> {
        let session = self.player_session().await?;
        self.players.add_player_dynamic_tag(&session, draft).await
    }
    pub async fn calculate_player_match_contribution(
        &self,
        request: PlayerMatchContributionRequest,
    ) -> ApplicationResult<PlayerMatchContribution> {
        let session = self.player_session().await?;
        self.players
            .calculate_player_match_contribution(&session, request)
            .await
    }
    pub async fn add_external_entity_id(
        &self,
        draft: ExternalEntityIdDraft,
    ) -> ApplicationResult<ExternalEntityIdRecord> {
        let session = self.player_session().await?;
        self.players.add_external_entity_id(&session, draft).await
    }
}
