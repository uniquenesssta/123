use crate::ports::PortResult;
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
pub trait FormationPort: Send + Sync {
    async fn list_formations(&self, active_only: bool) -> PortResult<Vec<FormationRecord>>;
    async fn save_usage_distribution(
        &self,
        draft: &FormationUsageDistributionDraft,
    ) -> PortResult<FormationUsageDistributionRecord>;
    async fn list_usage_distributions(
        &self,
        query: &FormationUsageListQuery,
    ) -> PortResult<Vec<FormationUsageDistributionRecord>>;
    async fn resolve_distribution(
        &self,
        query: &FormationDistributionQuery,
    ) -> PortResult<ResolvedFormationDistribution>;
}

#[async_trait]
pub trait MatchCatalogPort: Send + Sync {
    async fn create_match(&self, draft: &MatchDraft) -> PortResult<MatchRecord>;
    async fn delete_match(&self, match_id: Uuid) -> PortResult<()>;
    async fn read_match(&self, match_id: Uuid) -> PortResult<MatchRecord>;
}

#[async_trait]
pub trait LineupPort: Send + Sync {
    async fn create_lineup(&self, draft: &LineupDraft) -> PortResult<LineupRecord>;
    async fn create_lineup_pair(&self, draft: &LineupPairDraft) -> PortResult<LineupPairRecord>;
    async fn list_lineups(
        &self,
        match_id: Option<Uuid>,
        limit: u32,
    ) -> PortResult<Vec<LineupRecord>>;
    async fn read_lineup(&self, lineup_id: Uuid) -> PortResult<LineupRecord>;
    async fn remove_history(
        &self,
        lineup_id: Uuid,
        reason: Option<&str>,
    ) -> PortResult<LineupHistoryRemovalResult>;
    async fn read_match_chain(
        &self,
        match_id: Uuid,
        snapshot_type: &str,
    ) -> PortResult<MatchLineupChain>;
    async fn read_match_chain_at(
        &self,
        match_id: Uuid,
        snapshot_type: &str,
        reference_time: DateTime<Utc>,
    ) -> PortResult<MatchLineupChain>;
    async fn list_team_match_lineups(
        &self,
        team_id: Uuid,
        limit: u32,
    ) -> PortResult<Vec<TeamMatchLineupHistoryItem>>;
}

#[async_trait]
pub trait LineupPresetPort: Send + Sync {
    async fn save_preset(
        &self,
        draft: &TeamLineupPresetDraft,
    ) -> PortResult<TeamLineupPresetRecord>;
    async fn list_presets(
        &self,
        team_id: Uuid,
        include_archived: bool,
    ) -> PortResult<Vec<TeamLineupPresetRecord>>;
    async fn preview_application(
        &self,
        preset_id: Uuid,
    ) -> PortResult<TeamLineupPresetApplicationPreview>;
    async fn duplicate_preset(
        &self,
        preset_id: Uuid,
        name: &str,
    ) -> PortResult<TeamLineupPresetRecord>;
    async fn archive_preset(&self, preset_id: Uuid) -> PortResult<TeamLineupPresetRecord>;
    async fn delete_preset(&self, preset_id: Uuid) -> PortResult<()>;
}
