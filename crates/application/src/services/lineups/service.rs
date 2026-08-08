use crate::{
    ports::lineup::{FormationPort, LineupPort, LineupPresetPort, MatchCatalogPort},
    use_cases::lineups::{
        archive_team_lineup_preset, create_lineup, create_lineup_pair, create_match,
        delete_match, delete_team_lineup_preset, duplicate_team_lineup_preset, list_formations,
        list_formation_usage_distributions, list_lineups, list_team_lineup_presets,
        list_team_match_lineups, preview_team_lineup_preset_application, read_lineup,
        read_match_lineup_chain, remove_lineup_history, resolve_formation_distribution,
        save_formation_usage_distribution, save_team_lineup_preset,
    },
    ApplicationResult,
};
use football_domain::{
    FormationDistributionQuery, FormationRecord, FormationUsageDistributionDraft,
    FormationUsageDistributionRecord, FormationUsageListQuery, LineupDraft,
    LineupHistoryRemovalResult, LineupPairDraft, LineupPairRecord, LineupRecord, MatchDraft,
    MatchLineupChain, MatchRecord, ResolvedFormationDistribution,
    TeamLineupPresetApplicationPreview, TeamLineupPresetDraft, TeamLineupPresetRecord,
    TeamMatchLineupHistoryItem,
};
use uuid::Uuid;

pub(crate) struct LineupService;
impl LineupService {
    pub(crate) fn new() -> Self {
        Self
    }
    pub(crate) async fn list_formations<P: FormationPort + ?Sized>(
        &self,
        port: &P,
        active_only: bool,
    ) -> ApplicationResult<Vec<FormationRecord>> {
        list_formations::execute(port, active_only).await
    }
    pub(crate) async fn save_formation_usage_distribution<P: FormationPort + ?Sized>(
        &self,
        port: &P,
        draft: FormationUsageDistributionDraft,
    ) -> ApplicationResult<FormationUsageDistributionRecord> {
        save_formation_usage_distribution::execute(port, draft).await
    }
    pub(crate) async fn list_formation_usage_distributions<P: FormationPort + ?Sized>(
        &self,
        port: &P,
        query: FormationUsageListQuery,
    ) -> ApplicationResult<Vec<FormationUsageDistributionRecord>> {
        list_formation_usage_distributions::execute(port, query).await
    }
    pub(crate) async fn resolve_formation_distribution<P: FormationPort + ?Sized>(
        &self,
        port: &P,
        query: FormationDistributionQuery,
    ) -> ApplicationResult<ResolvedFormationDistribution> {
        resolve_formation_distribution::execute(port, query).await
    }
    pub(crate) async fn create_match<P: MatchCatalogPort + ?Sized>(
        &self,
        port: &P,
        draft: MatchDraft,
    ) -> ApplicationResult<MatchRecord> {
        create_match::execute(port, draft).await
    }
    pub(crate) async fn delete_match<P: MatchCatalogPort + ?Sized>(
        &self,
        port: &P,
        match_id: Uuid,
    ) -> ApplicationResult<()> {
        delete_match::execute(port, match_id).await
    }
    pub(crate) async fn save_team_lineup_preset<P: LineupPresetPort + ?Sized>(
        &self,
        port: &P,
        draft: TeamLineupPresetDraft,
    ) -> ApplicationResult<TeamLineupPresetRecord> {
        save_team_lineup_preset::execute(port, draft).await
    }
    pub(crate) async fn list_team_lineup_presets<P: LineupPresetPort + ?Sized>(
        &self,
        port: &P,
        team_id: Uuid,
        include_archived: bool,
    ) -> ApplicationResult<Vec<TeamLineupPresetRecord>> {
        list_team_lineup_presets::execute(port, team_id, include_archived).await
    }
    pub(crate) async fn preview_team_lineup_preset_application<P: LineupPresetPort + ?Sized>(
        &self,
        port: &P,
        preset_id: Uuid,
    ) -> ApplicationResult<TeamLineupPresetApplicationPreview> {
        preview_team_lineup_preset_application::execute(port, preset_id).await
    }
    pub(crate) async fn duplicate_team_lineup_preset<P: LineupPresetPort + ?Sized>(
        &self,
        port: &P,
        preset_id: Uuid,
        name: String,
    ) -> ApplicationResult<TeamLineupPresetRecord> {
        duplicate_team_lineup_preset::execute(port, preset_id, name).await
    }
    pub(crate) async fn archive_team_lineup_preset<P: LineupPresetPort + ?Sized>(
        &self,
        port: &P,
        preset_id: Uuid,
    ) -> ApplicationResult<TeamLineupPresetRecord> {
        archive_team_lineup_preset::execute(port, preset_id).await
    }
    pub(crate) async fn delete_team_lineup_preset<P: LineupPresetPort + ?Sized>(
        &self,
        port: &P,
        preset_id: Uuid,
    ) -> ApplicationResult<()> {
        delete_team_lineup_preset::execute(port, preset_id).await
    }
    pub(crate) async fn create_lineup<P: LineupPort + ?Sized>(
        &self,
        port: &P,
        draft: LineupDraft,
    ) -> ApplicationResult<LineupRecord> {
        create_lineup::execute(port, draft).await
    }
    pub(crate) async fn create_lineup_pair<P: LineupPort + ?Sized>(
        &self,
        port: &P,
        draft: LineupPairDraft,
    ) -> ApplicationResult<LineupPairRecord> {
        create_lineup_pair::execute(port, draft).await
    }
    pub(crate) async fn list_lineups<P: LineupPort + ?Sized>(
        &self,
        port: &P,
        match_id: Option<Uuid>,
        limit: u32,
    ) -> ApplicationResult<Vec<LineupRecord>> {
        list_lineups::execute(port, match_id, limit).await
    }
    pub(crate) async fn read_lineup<P: LineupPort + ?Sized>(
        &self,
        port: &P,
        lineup_id: Uuid,
    ) -> ApplicationResult<LineupRecord> {
        read_lineup::execute(port, lineup_id).await
    }
    pub(crate) async fn remove_lineup_history<P: LineupPort + ?Sized>(
        &self,
        port: &P,
        lineup_id: Uuid,
        reason: Option<String>,
    ) -> ApplicationResult<LineupHistoryRemovalResult> {
        remove_lineup_history::execute(port, lineup_id, reason).await
    }
    pub(crate) async fn read_match_lineup_chain<P: LineupPort + ?Sized>(
        &self,
        port: &P,
        match_id: Uuid,
        snapshot_type: String,
    ) -> ApplicationResult<MatchLineupChain> {
        read_match_lineup_chain::execute(port, match_id, snapshot_type).await
    }
    pub(crate) async fn list_team_match_lineups<P: LineupPort + ?Sized>(
        &self,
        port: &P,
        team_id: Uuid,
        limit: u32,
    ) -> ApplicationResult<Vec<TeamMatchLineupHistoryItem>> {
        list_team_match_lineups::execute(port, team_id, limit).await
    }
}
