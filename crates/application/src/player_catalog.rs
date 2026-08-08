use crate::{ApplicationResult, ApplicationService};
use football_domain::{
    FormationDistributionQuery, FormationRecord, FormationUsageDistributionDraft,
    FormationUsageDistributionRecord, FormationUsageListQuery, LineupDraft,
    LineupHistoryRemovalResult, LineupPairDraft, LineupPairRecord, LineupRecord, MatchDraft,
    MatchLineupChain, MatchRecord, TeamLineupPresetApplicationPreview, TeamLineupPresetDraft,
    TeamLineupPresetRecord, TeamMatchLineupHistoryItem,
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
