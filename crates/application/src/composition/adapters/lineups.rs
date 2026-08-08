use super::super::port_registry::{map_persistence_error, ActiveDatabase};
use crate::ports::{
    lineup::{FormationPort, LineupPort, LineupPresetPort, MatchCatalogPort},
    PortResult,
};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use football_domain::{
    FormationDistributionQuery, FormationRecord, FormationUsageDistributionDraft,
    FormationUsageDistributionRecord, FormationUsageListQuery, LineupDraft,
    LineupHistoryRemovalResult, LineupPairDraft, LineupPairRecord, LineupRecord, MatchDraft,
    MatchLineupChain, MatchRecord, ResolvedFormationDistribution,
    TeamLineupPresetApplicationPreview, TeamLineupPresetDraft, TeamLineupPresetRecord,
    TeamMatchLineupHistoryItem,
};
use uuid::Uuid;

#[async_trait]
impl FormationPort for ActiveDatabase {
    async fn list_formations(&self, active_only: bool) -> PortResult<Vec<FormationRecord>> {
        self.transition_store()
            .list_formations(active_only)
            .await
            .map_err(map_persistence_error)
    }
    async fn save_usage_distribution(
        &self,
        draft: &FormationUsageDistributionDraft,
    ) -> PortResult<FormationUsageDistributionRecord> {
        self.transition_store()
            .save_formation_usage_distribution(draft)
            .await
            .map_err(map_persistence_error)
    }
    async fn list_usage_distributions(
        &self,
        query: &FormationUsageListQuery,
    ) -> PortResult<Vec<FormationUsageDistributionRecord>> {
        self.transition_store()
            .list_formation_usage_distributions(query)
            .await
            .map_err(map_persistence_error)
    }
    async fn resolve_distribution(
        &self,
        query: &FormationDistributionQuery,
    ) -> PortResult<ResolvedFormationDistribution> {
        self.transition_store()
            .resolve_formation_distribution(query)
            .await
            .map_err(map_persistence_error)
    }
}

#[async_trait]
impl MatchCatalogPort for ActiveDatabase {
    async fn create_match(&self, draft: &MatchDraft) -> PortResult<MatchRecord> {
        self.transition_store()
            .create_match(draft)
            .await
            .map_err(map_persistence_error)
    }
    async fn delete_match(&self, match_id: Uuid) -> PortResult<()> {
        self.transition_store()
            .delete_match(match_id)
            .await
            .map_err(map_persistence_error)
    }
    async fn read_match(&self, match_id: Uuid) -> PortResult<MatchRecord> {
        self.transition_store()
            .read_match_exchange(match_id)
            .await
            .map_err(map_persistence_error)
    }
}

#[async_trait]
impl LineupPort for ActiveDatabase {
    async fn create_lineup(&self, draft: &LineupDraft) -> PortResult<LineupRecord> {
        self.transition_store()
            .create_lineup(draft)
            .await
            .map_err(map_persistence_error)
    }
    async fn create_lineup_pair(&self, draft: &LineupPairDraft) -> PortResult<LineupPairRecord> {
        self.transition_store()
            .create_lineup_pair(draft)
            .await
            .map_err(map_persistence_error)
    }
    async fn list_lineups(
        &self,
        match_id: Option<Uuid>,
        limit: u32,
    ) -> PortResult<Vec<LineupRecord>> {
        self.transition_store()
            .list_lineups(match_id, limit)
            .await
            .map_err(map_persistence_error)
    }
    async fn read_lineup(&self, lineup_id: Uuid) -> PortResult<LineupRecord> {
        self.transition_store()
            .read_lineup(lineup_id)
            .await
            .map_err(map_persistence_error)
    }
    async fn remove_history(
        &self,
        lineup_id: Uuid,
        reason: Option<&str>,
    ) -> PortResult<LineupHistoryRemovalResult> {
        self.transition_store()
            .remove_lineup_history(lineup_id, reason)
            .await
            .map_err(map_persistence_error)
    }
    async fn read_match_chain(
        &self,
        match_id: Uuid,
        snapshot_type: &str,
    ) -> PortResult<MatchLineupChain> {
        self.transition_store()
            .read_match_lineup_chain(match_id, snapshot_type)
            .await
            .map_err(map_persistence_error)
    }
    async fn read_match_chain_at(
        &self,
        match_id: Uuid,
        snapshot_type: &str,
        reference_time: DateTime<Utc>,
    ) -> PortResult<MatchLineupChain> {
        self.transition_store()
            .read_match_lineup_chain_at(match_id, snapshot_type, reference_time)
            .await
            .map_err(map_persistence_error)
    }
    async fn list_team_match_lineups(
        &self,
        team_id: Uuid,
        limit: u32,
    ) -> PortResult<Vec<TeamMatchLineupHistoryItem>> {
        self.transition_store()
            .list_team_match_lineups(team_id, limit)
            .await
            .map_err(map_persistence_error)
    }
}

#[async_trait]
impl LineupPresetPort for ActiveDatabase {
    async fn save_preset(
        &self,
        draft: &TeamLineupPresetDraft,
    ) -> PortResult<TeamLineupPresetRecord> {
        self.transition_store()
            .save_team_lineup_preset(draft)
            .await
            .map_err(map_persistence_error)
    }
    async fn list_presets(
        &self,
        team_id: Uuid,
        include_archived: bool,
    ) -> PortResult<Vec<TeamLineupPresetRecord>> {
        self.transition_store()
            .list_team_lineup_presets(team_id, include_archived)
            .await
            .map_err(map_persistence_error)
    }
    async fn preview_application(
        &self,
        preset_id: Uuid,
    ) -> PortResult<TeamLineupPresetApplicationPreview> {
        self.transition_store()
            .preview_team_lineup_preset_application(preset_id)
            .await
            .map_err(map_persistence_error)
    }
    async fn duplicate_preset(
        &self,
        preset_id: Uuid,
        name: &str,
    ) -> PortResult<TeamLineupPresetRecord> {
        self.transition_store()
            .duplicate_team_lineup_preset(preset_id, name)
            .await
            .map_err(map_persistence_error)
    }
    async fn archive_preset(&self, preset_id: Uuid) -> PortResult<TeamLineupPresetRecord> {
        self.transition_store()
            .archive_team_lineup_preset(preset_id)
            .await
            .map_err(map_persistence_error)
    }
    async fn delete_preset(&self, preset_id: Uuid) -> PortResult<()> {
        self.transition_store()
            .delete_team_lineup_preset(preset_id)
            .await
            .map_err(map_persistence_error)
    }
}
