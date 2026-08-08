use crate::{ApplicationError, ApplicationResult, ApplicationService};
use football_domain::{
    FormationDistributionQuery, FormationRecord, FormationUsageDistributionDraft,
    FormationUsageDistributionRecord, FormationUsageListQuery, LineupDraft,
    LineupHistoryRemovalResult, LineupPairDraft, LineupPairRecord, LineupRecord, MatchDraft,
    MatchLineupChain, MatchRecord, TeamLineupPresetApplicationPreview, TeamLineupPresetDraft,
    TeamLineupPresetRecord, TeamMatchLineupHistoryItem,
};
use uuid::Uuid;

impl ApplicationService {
    async fn lineup_session(&self) -> ApplicationResult<crate::composition::ActiveDatabase> {
        self.database
            .active_session()
            .await
            .ok_or(ApplicationError::DatabaseNotConnected)
    }
    pub async fn list_formations(
        &self,
        active_only: bool,
    ) -> ApplicationResult<Vec<FormationRecord>> {
        let session = self.lineup_session().await?;
        self.lineups.list_formations(&session, active_only).await
    }
    pub async fn save_formation_usage_distribution(
        &self,
        draft: FormationUsageDistributionDraft,
    ) -> ApplicationResult<FormationUsageDistributionRecord> {
        let session = self.lineup_session().await?;
        self.lineups
            .save_formation_usage_distribution(&session, draft)
            .await
    }
    pub async fn list_formation_usage_distributions(
        &self,
        query: FormationUsageListQuery,
    ) -> ApplicationResult<Vec<FormationUsageDistributionRecord>> {
        let session = self.lineup_session().await?;
        self.lineups
            .list_formation_usage_distributions(&session, query)
            .await
    }
    pub async fn resolve_formation_distribution(
        &self,
        query: FormationDistributionQuery,
    ) -> ApplicationResult<football_domain::ResolvedFormationDistribution> {
        let session = self.lineup_session().await?;
        self.lineups
            .resolve_formation_distribution(&session, query)
            .await
    }
    pub async fn create_match(&self, draft: MatchDraft) -> ApplicationResult<MatchRecord> {
        let session = self.lineup_session().await?;
        self.lineups.create_match(&session, draft).await
    }
    pub async fn delete_match(&self, match_id: Uuid) -> ApplicationResult<()> {
        let session = self.lineup_session().await?;
        self.lineups.delete_match(&session, match_id).await
    }
    pub async fn save_team_lineup_preset(
        &self,
        draft: TeamLineupPresetDraft,
    ) -> ApplicationResult<TeamLineupPresetRecord> {
        let session = self.lineup_session().await?;
        self.lineups.save_team_lineup_preset(&session, draft).await
    }
    pub async fn list_team_lineup_presets(
        &self,
        team_id: Uuid,
        include_archived: bool,
    ) -> ApplicationResult<Vec<TeamLineupPresetRecord>> {
        let session = self.lineup_session().await?;
        self.lineups
            .list_team_lineup_presets(&session, team_id, include_archived)
            .await
    }
    pub async fn preview_team_lineup_preset_application(
        &self,
        preset_id: Uuid,
    ) -> ApplicationResult<TeamLineupPresetApplicationPreview> {
        let session = self.lineup_session().await?;
        self.lineups
            .preview_team_lineup_preset_application(&session, preset_id)
            .await
    }
    pub async fn duplicate_team_lineup_preset(
        &self,
        preset_id: Uuid,
        name: String,
    ) -> ApplicationResult<TeamLineupPresetRecord> {
        let session = self.lineup_session().await?;
        self.lineups
            .duplicate_team_lineup_preset(&session, preset_id, name)
            .await
    }
    pub async fn archive_team_lineup_preset(
        &self,
        preset_id: Uuid,
    ) -> ApplicationResult<TeamLineupPresetRecord> {
        let session = self.lineup_session().await?;
        self.lineups
            .archive_team_lineup_preset(&session, preset_id)
            .await
    }
    pub async fn delete_team_lineup_preset(&self, preset_id: Uuid) -> ApplicationResult<()> {
        let session = self.lineup_session().await?;
        self.lineups
            .delete_team_lineup_preset(&session, preset_id)
            .await
    }
    pub async fn create_lineup(&self, draft: LineupDraft) -> ApplicationResult<LineupRecord> {
        let session = self.lineup_session().await?;
        self.lineups.create_lineup(&session, draft).await
    }
    pub async fn create_lineup_pair(
        &self,
        draft: LineupPairDraft,
    ) -> ApplicationResult<LineupPairRecord> {
        let session = self.lineup_session().await?;
        self.lineups.create_lineup_pair(&session, draft).await
    }
    pub async fn list_lineups(
        &self,
        match_id: Option<Uuid>,
        limit: u32,
    ) -> ApplicationResult<Vec<LineupRecord>> {
        let session = self.lineup_session().await?;
        self.lineups.list_lineups(&session, match_id, limit).await
    }
    pub async fn read_lineup(&self, lineup_id: Uuid) -> ApplicationResult<LineupRecord> {
        let session = self.lineup_session().await?;
        self.lineups.read_lineup(&session, lineup_id).await
    }
    pub async fn remove_lineup_history(
        &self,
        lineup_id: Uuid,
        reason: Option<String>,
    ) -> ApplicationResult<LineupHistoryRemovalResult> {
        let session = self.lineup_session().await?;
        self.lineups
            .remove_lineup_history(&session, lineup_id, reason)
            .await
    }
    pub async fn read_match_lineup_chain(
        &self,
        match_id: Uuid,
        snapshot_type: String,
    ) -> ApplicationResult<MatchLineupChain> {
        let session = self.lineup_session().await?;
        self.lineups
            .read_match_lineup_chain(&session, match_id, snapshot_type)
            .await
    }
    pub async fn list_team_match_lineups(
        &self,
        team_id: Uuid,
        limit: u32,
    ) -> ApplicationResult<Vec<TeamMatchLineupHistoryItem>> {
        let session = self.lineup_session().await?;
        self.lineups
            .list_team_match_lineups(&session, team_id, limit)
            .await
    }
}
