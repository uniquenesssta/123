use super::super::port_registry::PersistenceStore;
use super::map_persistence_error;
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
impl FormationPort for PersistenceStore {
    async fn list_formations(&self, active_only: bool) -> PortResult<Vec<FormationRecord>> {
        self.list_formations(active_only)
            .await
            .map_err(map_persistence_error)
    }
    async fn save_usage_distribution(
        &self,
        draft: &FormationUsageDistributionDraft,
    ) -> PortResult<FormationUsageDistributionRecord> {
        self.save_formation_usage_distribution(draft)
            .await
            .map_err(map_persistence_error)
    }
    async fn list_usage_distributions(
        &self,
        query: &FormationUsageListQuery,
    ) -> PortResult<Vec<FormationUsageDistributionRecord>> {
        self.list_formation_usage_distributions(query)
            .await
            .map_err(map_persistence_error)
    }
    async fn resolve_distribution(
        &self,
        query: &FormationDistributionQuery,
    ) -> PortResult<ResolvedFormationDistribution> {
        self.resolve_formation_distribution(query)
            .await
            .map_err(map_persistence_error)
    }
}

#[async_trait]
impl MatchCatalogPort for PersistenceStore {
    async fn create_match(&self, draft: &MatchDraft) -> PortResult<MatchRecord> {
        self.create_match(draft)
            .await
            .map_err(map_persistence_error)
    }
    async fn delete_match(&self, match_id: Uuid) -> PortResult<()> {
        self.delete_match(match_id)
            .await
            .map_err(map_persistence_error)
    }
    async fn read_match(&self, match_id: Uuid) -> PortResult<MatchRecord> {
        self.read_match_exchange(match_id)
            .await
            .map_err(map_persistence_error)
    }
}

#[async_trait]
impl LineupPort for PersistenceStore {
    async fn create_lineup(&self, draft: &LineupDraft) -> PortResult<LineupRecord> {
        self.create_lineup(draft)
            .await
            .map_err(map_persistence_error)
    }
    async fn create_lineup_pair(&self, draft: &LineupPairDraft) -> PortResult<LineupPairRecord> {
        self.create_lineup_pair(draft)
            .await
            .map_err(map_persistence_error)
    }
    async fn list_lineups(
        &self,
        match_id: Option<Uuid>,
        limit: u32,
    ) -> PortResult<Vec<LineupRecord>> {
        self.list_lineups(match_id, limit)
            .await
            .map_err(map_persistence_error)
    }
    async fn read_lineup(&self, lineup_id: Uuid) -> PortResult<LineupRecord> {
        self.read_lineup(lineup_id)
            .await
            .map_err(map_persistence_error)
    }
    async fn remove_history(
        &self,
        lineup_id: Uuid,
        reason: Option<&str>,
    ) -> PortResult<LineupHistoryRemovalResult> {
        self.remove_lineup_history(lineup_id, reason)
            .await
            .map_err(map_persistence_error)
    }
    async fn read_match_chain(
        &self,
        match_id: Uuid,
        snapshot_type: &str,
    ) -> PortResult<MatchLineupChain> {
        self.read_match_lineup_chain(match_id, snapshot_type)
            .await
            .map_err(map_persistence_error)
    }
    async fn read_match_chain_at(
        &self,
        match_id: Uuid,
        snapshot_type: &str,
        reference_time: DateTime<Utc>,
    ) -> PortResult<MatchLineupChain> {
        self.read_match_lineup_chain_at(match_id, snapshot_type, reference_time)
            .await
            .map_err(map_persistence_error)
    }
    async fn list_team_match_lineups(
        &self,
        team_id: Uuid,
        limit: u32,
    ) -> PortResult<Vec<TeamMatchLineupHistoryItem>> {
        self.list_team_match_lineups(team_id, limit)
            .await
            .map_err(map_persistence_error)
    }
}

#[async_trait]
impl LineupPresetPort for PersistenceStore {
    async fn save_preset(
        &self,
        draft: &TeamLineupPresetDraft,
    ) -> PortResult<TeamLineupPresetRecord> {
        self.save_team_lineup_preset(draft)
            .await
            .map_err(map_persistence_error)
    }
    async fn list_presets(
        &self,
        team_id: Uuid,
        include_archived: bool,
    ) -> PortResult<Vec<TeamLineupPresetRecord>> {
        self.list_team_lineup_presets(team_id, include_archived)
            .await
            .map_err(map_persistence_error)
    }
    async fn preview_application(
        &self,
        preset_id: Uuid,
    ) -> PortResult<TeamLineupPresetApplicationPreview> {
        self.preview_team_lineup_preset_application(preset_id)
            .await
            .map_err(map_persistence_error)
    }
    async fn duplicate_preset(
        &self,
        preset_id: Uuid,
        name: &str,
    ) -> PortResult<TeamLineupPresetRecord> {
        self.duplicate_team_lineup_preset(preset_id, name)
            .await
            .map_err(map_persistence_error)
    }
    async fn archive_preset(&self, preset_id: Uuid) -> PortResult<TeamLineupPresetRecord> {
        self.archive_team_lineup_preset(preset_id)
            .await
            .map_err(map_persistence_error)
    }
    async fn delete_preset(&self, preset_id: Uuid) -> PortResult<()> {
        self.delete_team_lineup_preset(preset_id)
            .await
            .map_err(map_persistence_error)
    }
}
