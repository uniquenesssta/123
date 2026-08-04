use crate::{ApplicationResult, ApplicationService};
use football_domain::{
    BulkArchiveResult, BulkDeleteResult, CoachDetail, CoachDraft, CoachListItem, CoachListQuery,
    CoachNameDraft, CoachNameRecord, CoachRecord, DataProviderDraft, DataProviderRecord,
    EntityDeletionCheck, EntityMatchRequest, EntityMatchResult, EntityReferenceQuery,
    EntityReferenceRecord, ExternalEntityIdDraft, ExternalEntityIdRecord,
    FormationDistributionQuery, FormationRecord, FormationUsageDistributionDraft,
    FormationUsageDistributionRecord, FormationUsageListQuery, LineupDraft,
    LineupHistoryRemovalResult, LineupPairDraft, LineupPairRecord, LineupRecord, MatchDraft,
    MatchLineupChain, MatchRecord, PlayerAbilityObservationDraft, PlayerAbilityObservationRecord,
    PlayerAvailabilityDraft, PlayerAvailabilityRecord, PlayerCatalogReferenceData, PlayerDetail,
    PlayerDraft, PlayerDynamicTagDraft, PlayerDynamicTagRecord, PlayerListPage, PlayerListQuery,
    PlayerMatchContribution, PlayerMatchContributionRequest, PlayerNameDraft, PlayerNameRecord,
    PlayerPositionDraft, PlayerPositionRecord, PlayerRecord, PlayerTeamPeriodDraft,
    PlayerTeamPeriodRecord, TeamCoachPeriodDraft, TeamCoachPeriodRecord, TeamDetail, TeamDraft,
    TeamForceDeletePreview, TeamForceDeleteRequest, TeamForceDeleteResult,
    TeamLineupPresetApplicationPreview, TeamLineupPresetDraft, TeamLineupPresetRecord,
    TeamListPage, TeamListQuery, TeamMatchLineupHistoryItem, TeamNameDraft, TeamNameRecord,
    TeamOption, TeamProfileDraft, TeamProfileRecord, TeamRecord,
};
use uuid::Uuid;

impl ApplicationService {
    pub async fn list_formations(
        &self,
        active_only: bool,
    ) -> ApplicationResult<Vec<FormationRecord>> {
        let store = self.active_store().await?;
        Ok(store.list_formations(active_only).await?)
    }

    pub async fn save_formation_usage_distribution(
        &self,
        draft: FormationUsageDistributionDraft,
    ) -> ApplicationResult<FormationUsageDistributionRecord> {
        let store = self.active_store().await?;
        Ok(store.save_formation_usage_distribution(&draft).await?)
    }

    pub async fn list_formation_usage_distributions(
        &self,
        query: FormationUsageListQuery,
    ) -> ApplicationResult<Vec<FormationUsageDistributionRecord>> {
        let store = self.active_store().await?;
        Ok(store.list_formation_usage_distributions(&query).await?)
    }

    pub async fn resolve_formation_distribution(
        &self,
        query: FormationDistributionQuery,
    ) -> ApplicationResult<football_domain::ResolvedFormationDistribution> {
        let store = self.active_store().await?;
        Ok(store.resolve_formation_distribution(&query).await?)
    }

    pub async fn player_catalog_reference_data(
        &self,
    ) -> ApplicationResult<PlayerCatalogReferenceData> {
        let store = self.active_store().await?;
        Ok(store.player_catalog_reference_data().await?)
    }

    pub async fn create_coach(&self, draft: CoachDraft) -> ApplicationResult<CoachRecord> {
        let store = self.active_store().await?;
        Ok(store.create_coach(&draft).await?)
    }

    pub async fn list_coaches(
        &self,
        query: CoachListQuery,
    ) -> ApplicationResult<Vec<CoachListItem>> {
        let store = self.active_store().await?;
        Ok(store.list_coaches(&query).await?)
    }

    pub async fn read_coach(&self, coach_id: Uuid) -> ApplicationResult<CoachDetail> {
        let store = self.active_store().await?;
        Ok(store.read_coach(coach_id).await?)
    }

    pub async fn add_coach_name(
        &self,
        draft: CoachNameDraft,
    ) -> ApplicationResult<CoachNameRecord> {
        let store = self.active_store().await?;
        Ok(store.add_coach_name(&draft).await?)
    }

    pub async fn add_team_coach_period(
        &self,
        draft: TeamCoachPeriodDraft,
    ) -> ApplicationResult<TeamCoachPeriodRecord> {
        let store = self.active_store().await?;
        Ok(store.add_team_coach_period(&draft).await?)
    }

    pub async fn list_entity_references(
        &self,
        query: EntityReferenceQuery,
    ) -> ApplicationResult<Vec<EntityReferenceRecord>> {
        let store = self.active_store().await?;
        Ok(store.list_entity_references(&query).await?)
    }

    pub async fn resolve_entity_reference(
        &self,
        request: EntityMatchRequest,
    ) -> ApplicationResult<EntityMatchResult> {
        let store = self.active_store().await?;
        Ok(store.resolve_entity_reference(&request).await?)
    }

    pub async fn check_entity_deletion(
        &self,
        entity_type: String,
        entity_id: Uuid,
    ) -> ApplicationResult<EntityDeletionCheck> {
        let store = self.active_store().await?;
        Ok(store.check_entity_deletion(&entity_type, entity_id).await?)
    }

    pub async fn bulk_archive_entities(
        &self,
        entity_type: String,
        entity_ids: Vec<Uuid>,
    ) -> ApplicationResult<BulkArchiveResult> {
        let store = self.active_store().await?;
        Ok(store
            .bulk_archive_entities(&entity_type, &entity_ids)
            .await?)
    }

    pub async fn create_team(&self, draft: TeamDraft) -> ApplicationResult<TeamRecord> {
        let store = self.active_store().await?;
        Ok(store.create_team(&draft).await?)
    }

    pub async fn list_team_options(
        &self,
        search: Option<String>,
        limit: u32,
    ) -> ApplicationResult<Vec<TeamOption>> {
        let store = self.active_store().await?;
        Ok(store.list_team_options(search.as_deref(), limit).await?)
    }

    pub async fn list_teams(&self, query: TeamListQuery) -> ApplicationResult<TeamListPage> {
        let store = self.active_store().await?;
        Ok(store.list_teams(&query).await?)
    }

    pub async fn read_team(&self, team_id: Uuid) -> ApplicationResult<TeamDetail> {
        let store = self.active_store().await?;
        Ok(store.read_team(team_id).await?)
    }

    pub async fn update_team(
        &self,
        team_id: Uuid,
        draft: TeamDraft,
    ) -> ApplicationResult<TeamRecord> {
        let store = self.active_store().await?;
        Ok(store.update_team(team_id, &draft).await?)
    }

    pub async fn add_team_name(&self, draft: TeamNameDraft) -> ApplicationResult<TeamNameRecord> {
        let store = self.active_store().await?;
        Ok(store.add_team_name(&draft).await?)
    }

    pub async fn upsert_team_profile(
        &self,
        team_id: Uuid,
        draft: TeamProfileDraft,
    ) -> ApplicationResult<TeamProfileRecord> {
        let store = self.active_store().await?;
        Ok(store.upsert_team_profile(team_id, &draft).await?)
    }

    pub async fn bulk_delete_players(
        &self,
        player_ids: Vec<Uuid>,
    ) -> ApplicationResult<BulkDeleteResult> {
        let store = self.active_store().await?;
        Ok(store.bulk_delete_players(&player_ids).await?)
    }

    pub async fn bulk_delete_teams(
        &self,
        team_ids: Vec<Uuid>,
    ) -> ApplicationResult<BulkDeleteResult> {
        let store = self.active_store().await?;
        Ok(store.bulk_delete_teams(&team_ids).await?)
    }

    pub async fn preview_force_delete_team(
        &self,
        team_id: Uuid,
    ) -> ApplicationResult<TeamForceDeletePreview> {
        let store = self.active_store().await?;
        Ok(store.preview_force_delete_team(team_id).await?)
    }

    pub async fn force_delete_team(
        &self,
        request: TeamForceDeleteRequest,
    ) -> ApplicationResult<TeamForceDeleteResult> {
        let store = self.active_store().await?;
        Ok(store.force_delete_team(&request).await?)
    }

    pub async fn create_data_provider(
        &self,
        draft: DataProviderDraft,
    ) -> ApplicationResult<DataProviderRecord> {
        let store = self.active_store().await?;
        Ok(store.create_data_provider(&draft).await?)
    }

    pub async fn create_player(&self, draft: PlayerDraft) -> ApplicationResult<PlayerRecord> {
        let store = self.active_store().await?;
        Ok(store.create_player(&draft).await?)
    }

    pub async fn update_player(
        &self,
        player_id: Uuid,
        draft: PlayerDraft,
    ) -> ApplicationResult<PlayerRecord> {
        let store = self.active_store().await?;
        Ok(store.update_player(player_id, &draft).await?)
    }

    pub async fn delete_player(&self, player_id: Uuid) -> ApplicationResult<()> {
        let store = self.active_store().await?;
        Ok(store.delete_player(player_id).await?)
    }

    pub async fn list_players(&self, query: PlayerListQuery) -> ApplicationResult<PlayerListPage> {
        let store = self.active_store().await?;
        Ok(store.list_players(&query).await?)
    }

    pub async fn read_player(&self, player_id: Uuid) -> ApplicationResult<PlayerDetail> {
        let store = self.active_store().await?;
        Ok(store.read_player(player_id).await?)
    }

    pub async fn add_player_name(
        &self,
        draft: PlayerNameDraft,
    ) -> ApplicationResult<PlayerNameRecord> {
        let store = self.active_store().await?;
        Ok(store.add_player_name(&draft).await?)
    }

    pub async fn assign_player_position(
        &self,
        draft: PlayerPositionDraft,
    ) -> ApplicationResult<PlayerPositionRecord> {
        let store = self.active_store().await?;
        Ok(store.assign_player_position(&draft).await?)
    }

    pub async fn add_player_team_period(
        &self,
        draft: PlayerTeamPeriodDraft,
    ) -> ApplicationResult<PlayerTeamPeriodRecord> {
        let allowed = ["registered", "loan", "trial", "released", "unknown"];
        if !allowed.contains(&draft.registration_status.as_str()) {
            return Err(crate::ApplicationError::Validation(format!(
                "未知注册状态：{}",
                draft.registration_status
            )));
        }
        let store = self.active_store().await?;
        Ok(store.add_player_team_period(&draft).await?)
    }

    pub async fn add_player_availability(
        &self,
        draft: PlayerAvailabilityDraft,
    ) -> ApplicationResult<PlayerAvailabilityRecord> {
        let store = self.active_store().await?;
        Ok(store.add_player_availability(&draft).await?)
    }

    pub async fn add_player_ability_observation(
        &self,
        draft: PlayerAbilityObservationDraft,
    ) -> ApplicationResult<PlayerAbilityObservationRecord> {
        if draft.calculation_version.trim().is_empty() {
            return Err(crate::ApplicationError::Validation(
                "能力观察必须提供 calculation_version".to_string(),
            ));
        }
        let store = self.active_store().await?;
        Ok(store.add_player_ability_observation(&draft).await?)
    }

    pub async fn add_player_dynamic_tag(
        &self,
        draft: PlayerDynamicTagDraft,
    ) -> ApplicationResult<PlayerDynamicTagRecord> {
        let store = self.active_store().await?;
        Ok(store.add_player_dynamic_tag(&draft).await?)
    }

    pub async fn calculate_player_match_contribution(
        &self,
        request: PlayerMatchContributionRequest,
    ) -> ApplicationResult<PlayerMatchContribution> {
        let store = self.active_store().await?;
        Ok(store.calculate_player_match_contribution(&request).await?)
    }

    pub async fn add_external_entity_id(
        &self,
        draft: ExternalEntityIdDraft,
    ) -> ApplicationResult<ExternalEntityIdRecord> {
        let store = self.active_store().await?;
        Ok(store.add_external_entity_id(&draft).await?)
    }

    pub async fn create_match(&self, draft: MatchDraft) -> ApplicationResult<MatchRecord> {
        let store = self.active_store().await?;
        Ok(store.create_match(&draft).await?)
    }

    pub async fn delete_match(&self, match_id: Uuid) -> ApplicationResult<()> {
        let store = self.active_store().await?;
        Ok(store.delete_match(match_id).await?)
    }

    pub async fn save_team_lineup_preset(
        &self,
        draft: TeamLineupPresetDraft,
    ) -> ApplicationResult<TeamLineupPresetRecord> {
        let store = self.active_store().await?;
        Ok(store.save_team_lineup_preset(&draft).await?)
    }

    pub async fn list_team_lineup_presets(
        &self,
        team_id: Uuid,
        include_archived: bool,
    ) -> ApplicationResult<Vec<TeamLineupPresetRecord>> {
        let store = self.active_store().await?;
        Ok(store
            .list_team_lineup_presets(team_id, include_archived)
            .await?)
    }

    pub async fn preview_team_lineup_preset_application(
        &self,
        preset_id: Uuid,
    ) -> ApplicationResult<TeamLineupPresetApplicationPreview> {
        let store = self.active_store().await?;
        Ok(store
            .preview_team_lineup_preset_application(preset_id)
            .await?)
    }

    pub async fn duplicate_team_lineup_preset(
        &self,
        preset_id: Uuid,
        name: String,
    ) -> ApplicationResult<TeamLineupPresetRecord> {
        let store = self.active_store().await?;
        Ok(store.duplicate_team_lineup_preset(preset_id, &name).await?)
    }

    pub async fn archive_team_lineup_preset(
        &self,
        preset_id: Uuid,
    ) -> ApplicationResult<TeamLineupPresetRecord> {
        let store = self.active_store().await?;
        Ok(store.archive_team_lineup_preset(preset_id).await?)
    }

    pub async fn delete_team_lineup_preset(&self, preset_id: Uuid) -> ApplicationResult<()> {
        let store = self.active_store().await?;
        Ok(store.delete_team_lineup_preset(preset_id).await?)
    }

    pub async fn create_lineup(&self, draft: LineupDraft) -> ApplicationResult<LineupRecord> {
        let store = self.active_store().await?;
        Ok(store.create_lineup(&draft).await?)
    }

    pub async fn create_lineup_pair(
        &self,
        draft: LineupPairDraft,
    ) -> ApplicationResult<LineupPairRecord> {
        let store = self.active_store().await?;
        Ok(store.create_lineup_pair(&draft).await?)
    }

    pub async fn list_lineups(
        &self,
        match_id: Option<Uuid>,
        limit: u32,
    ) -> ApplicationResult<Vec<LineupRecord>> {
        let store = self.active_store().await?;
        Ok(store.list_lineups(match_id, limit).await?)
    }

    pub async fn read_lineup(&self, lineup_id: Uuid) -> ApplicationResult<LineupRecord> {
        let store = self.active_store().await?;
        Ok(store.read_lineup(lineup_id).await?)
    }

    pub async fn remove_lineup_history(
        &self,
        lineup_id: Uuid,
        reason: Option<String>,
    ) -> ApplicationResult<LineupHistoryRemovalResult> {
        let store = self.active_store().await?;
        Ok(store
            .remove_lineup_history(lineup_id, reason.as_deref())
            .await?)
    }

    pub async fn read_match_lineup_chain(
        &self,
        match_id: Uuid,
        snapshot_type: String,
    ) -> ApplicationResult<MatchLineupChain> {
        let store = self.active_store().await?;
        Ok(store
            .read_match_lineup_chain(match_id, &snapshot_type)
            .await?)
    }

    pub async fn list_team_match_lineups(
        &self,
        team_id: Uuid,
        limit: u32,
    ) -> ApplicationResult<Vec<TeamMatchLineupHistoryItem>> {
        let store = self.active_store().await?;
        Ok(store.list_team_match_lineups(team_id, limit).await?)
    }
}
