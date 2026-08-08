pub(crate) use football_persistence_postgres::{
    DatabaseHealth, DatabaseOptions, DatabaseStats, ModelRunListItem, PersistenceError,
    PostgresStore as PersistenceStore,
};

use crate::ports::{
    competition::CompetitionHierarchyPort,
    database::{
        DatabaseHealthSnapshot, DatabaseLifecyclePort, DatabaseObservabilityPort,
        DatabaseStatistics,
    },
    player::{CoachCatalogPort, EntityReferencePort, PlayerCatalogPort, PlayerSignalPort},
    rules::{RulePackagePort, RuleRoutingPort},
    team::{TeamCatalogPort, TeamLifecyclePort},
    PortError, PortErrorKind, PortResult,
};
use async_trait::async_trait;
use football_domain::{
    BulkArchiveResult, BulkDeleteResult, CoachDetail, CoachDraft, CoachListItem, CoachListQuery,
    CoachNameDraft, CoachNameRecord, CoachRecord, CompetitionBindingDraft,
    CompetitionBindingSummary, CompetitionDraft, CompetitionKind, CompetitionRecord,
    DataProviderDraft, DataProviderRecord, EntityDeletionCheck, EntityMatchRequest,
    EntityMatchResult, EntityReferenceQuery, EntityReferenceRecord, ExternalEntityIdDraft,
    ExternalEntityIdRecord, PlayerAbilityObservationDraft, PlayerAbilityObservationRecord,
    PlayerAvailabilityDraft, PlayerAvailabilityRecord, PlayerCatalogReferenceData, PlayerDetail,
    PlayerDraft, PlayerDynamicTagDraft, PlayerDynamicTagRecord, PlayerListPage, PlayerListQuery,
    PlayerMatchContribution, PlayerMatchContributionRequest, PlayerNameDraft, PlayerNameRecord,
    PlayerPositionDraft, PlayerPositionRecord, PlayerRecord, PlayerTeamPeriodDraft,
    PlayerTeamPeriodRecord, RoundDraft, RoundRecord, RouteDecision, RouteRequest, RulePackageDraft,
    RulePackageSummary, SeasonDraft, SeasonRecord, StageDraft, StageRecord, TeamCoachPeriodDraft,
    TeamCoachPeriodRecord, TeamDetail, TeamDraft, TeamForceDeletePreview, TeamForceDeleteRequest,
    TeamForceDeleteResult, TeamListPage, TeamListQuery, TeamNameDraft, TeamNameRecord, TeamOption,
    TeamProfileDraft, TeamProfileRecord, TeamRecord,
};
use football_model_api::ModelDescriptor;
use uuid::Uuid;

#[derive(Clone)]
pub(crate) struct ActiveDatabase {
    store: PersistenceStore,
    redacted_url: String,
}

impl ActiveDatabase {
    pub(crate) async fn connect(options: &DatabaseOptions) -> PortResult<Self> {
        let store = PersistenceStore::connect(options)
            .await
            .map_err(map_persistence_error)?;
        Ok(Self {
            store,
            redacted_url: options.redacted_url(),
        })
    }

    pub(crate) fn redacted_url(&self) -> &str {
        &self.redacted_url
    }

    pub(crate) fn transition_store(&self) -> PersistenceStore {
        self.store.clone()
    }
}

pub(crate) struct PortRegistry;

impl PortRegistry {
    pub(crate) fn new() -> Self {
        Self
    }

    pub(crate) async fn connect_database(
        &self,
        options: &DatabaseOptions,
    ) -> PortResult<ActiveDatabase> {
        ActiveDatabase::connect(options).await
    }
}

pub(crate) fn database_health_from_snapshot(snapshot: DatabaseHealthSnapshot) -> DatabaseHealth {
    DatabaseHealth {
        connected: snapshot.connected,
        database_name: snapshot.database_name,
        server_version: snapshot.server_version,
        migration_count: snapshot.migration_count,
        database_size_bytes: snapshot.database_size_bytes,
        checked_at: snapshot.checked_at,
        latency_ms: snapshot.latency_ms,
    }
}

pub(crate) fn database_stats_from_statistics(statistics: DatabaseStatistics) -> DatabaseStats {
    DatabaseStats {
        competitions: statistics.competitions,
        teams: statistics.teams,
        players: statistics.players,
        matches: statistics.matches,
        model_runs: statistics.model_runs,
        rule_packages: statistics.rule_packages,
        route_bindings: statistics.route_bindings,
        ability_observations: statistics.ability_observations,
        pending_ability_updates: statistics.pending_ability_updates,
        data_providers: statistics.data_providers,
        availability_records: statistics.availability_records,
        active_lineups: statistics.active_lineups,
        large_counts_are_estimates: statistics.large_counts_are_estimates,
    }
}

fn map_persistence_error(error: PersistenceError) -> PortError {
    let kind = match &error {
        PersistenceError::Serialization(_) => PortErrorKind::Serialization,
        PersistenceError::InvalidState(_) => PortErrorKind::InvalidState,
        PersistenceError::RouteNotFound => PortErrorKind::NotFound,
        PersistenceError::Sqlx(_) | PersistenceError::Migration(_) => PortErrorKind::Infrastructure,
    };
    PortError::new(kind, error.to_string())
}

#[async_trait]
impl DatabaseLifecyclePort for ActiveDatabase {
    async fn migrate(&self) -> PortResult<()> {
        self.store.migrate().await.map_err(map_persistence_error)
    }

    async fn recover_interrupted_work(&self) -> PortResult<()> {
        self.store
            .recover_interrupted_jobs()
            .await
            .map_err(map_persistence_error)?;
        self.store
            .recover_interrupted_api_workspace_operations()
            .await
            .map_err(map_persistence_error)?;
        Ok(())
    }

    async fn reset_to_pristine(&self) -> PortResult<()> {
        self.store
            .reset_to_pristine()
            .await
            .map_err(map_persistence_error)
    }

    async fn close(&self) -> PortResult<()> {
        self.store.close().await;
        Ok(())
    }
}

#[async_trait]
impl DatabaseObservabilityPort for ActiveDatabase {
    async fn health(&self) -> PortResult<DatabaseHealthSnapshot> {
        let health = self.store.health().await.map_err(map_persistence_error)?;
        Ok(DatabaseHealthSnapshot {
            connected: health.connected,
            database_name: health.database_name,
            server_version: health.server_version,
            migration_count: health.migration_count,
            database_size_bytes: health.database_size_bytes,
            checked_at: health.checked_at,
            latency_ms: health.latency_ms,
        })
    }

    async fn statistics(&self) -> PortResult<DatabaseStatistics> {
        let statistics = self.store.stats().await.map_err(map_persistence_error)?;
        Ok(DatabaseStatistics {
            competitions: statistics.competitions,
            teams: statistics.teams,
            players: statistics.players,
            matches: statistics.matches,
            model_runs: statistics.model_runs,
            rule_packages: statistics.rule_packages,
            route_bindings: statistics.route_bindings,
            ability_observations: statistics.ability_observations,
            pending_ability_updates: statistics.pending_ability_updates,
            data_providers: statistics.data_providers,
            availability_records: statistics.availability_records,
            active_lineups: statistics.active_lineups,
            large_counts_are_estimates: statistics.large_counts_are_estimates,
        })
    }
}

#[async_trait]
impl CompetitionHierarchyPort for ActiveDatabase {
    async fn create_competition(&self, draft: &CompetitionDraft) -> PortResult<CompetitionRecord> {
        self.store
            .create_competition(draft)
            .await
            .map_err(map_persistence_error)
    }

    async fn delete_competition(&self, competition_id: Uuid) -> PortResult<()> {
        self.store
            .delete_competition(competition_id)
            .await
            .map_err(map_persistence_error)
    }

    async fn list_competitions(&self) -> PortResult<Vec<CompetitionRecord>> {
        self.store
            .list_competitions()
            .await
            .map_err(map_persistence_error)
    }

    async fn create_season(&self, draft: &SeasonDraft) -> PortResult<SeasonRecord> {
        self.store
            .create_season(draft)
            .await
            .map_err(map_persistence_error)
    }

    async fn list_seasons(&self) -> PortResult<Vec<SeasonRecord>> {
        self.store
            .list_seasons()
            .await
            .map_err(map_persistence_error)
    }

    async fn create_stage(&self, draft: &StageDraft) -> PortResult<StageRecord> {
        self.store
            .create_stage(draft)
            .await
            .map_err(map_persistence_error)
    }

    async fn list_stages(&self) -> PortResult<Vec<StageRecord>> {
        self.store
            .list_stages()
            .await
            .map_err(map_persistence_error)
    }

    async fn create_round(&self, draft: &RoundDraft) -> PortResult<RoundRecord> {
        self.store
            .create_round(draft)
            .await
            .map_err(map_persistence_error)
    }

    async fn list_rounds(&self) -> PortResult<Vec<RoundRecord>> {
        self.store
            .list_rounds()
            .await
            .map_err(map_persistence_error)
    }
}

#[async_trait]
impl RulePackagePort for ActiveDatabase {
    async fn register_rule_package(
        &self,
        descriptor: &ModelDescriptor,
        draft: &RulePackageDraft,
    ) -> PortResult<RulePackageSummary> {
        self.store
            .register_rule_package(descriptor, draft)
            .await
            .map_err(map_persistence_error)
    }

    async fn list_rule_packages(&self) -> PortResult<Vec<RulePackageSummary>> {
        self.store
            .list_rule_packages()
            .await
            .map_err(map_persistence_error)
    }
}

#[async_trait]
impl RuleRoutingPort for ActiveDatabase {
    async fn create_competition_binding(
        &self,
        draft: &CompetitionBindingDraft,
    ) -> PortResult<CompetitionBindingSummary> {
        self.store
            .create_competition_binding(draft)
            .await
            .map_err(map_persistence_error)
    }

    async fn list_competition_bindings(&self) -> PortResult<Vec<CompetitionBindingSummary>> {
        self.store
            .list_competition_bindings()
            .await
            .map_err(map_persistence_error)
    }

    async fn ensure_type_default_binding(
        &self,
        rule_package_id: Uuid,
        competition_kind: CompetitionKind,
        priority: i32,
        label: &str,
    ) -> PortResult<()> {
        self.store
            .ensure_type_default_binding(rule_package_id, competition_kind, priority, label)
            .await
            .map(|_| ())
            .map_err(map_persistence_error)
    }

    async fn resolve_route(&self, request: &RouteRequest) -> PortResult<RouteDecision> {
        self.store
            .resolve_route(request)
            .await
            .map_err(map_persistence_error)
    }
}

#[async_trait]
impl TeamCatalogPort for ActiveDatabase {
    async fn create_team(&self, draft: &TeamDraft) -> PortResult<TeamRecord> {
        self.store
            .create_team(draft)
            .await
            .map_err(map_persistence_error)
    }
    async fn list_team_options(
        &self,
        search: Option<&str>,
        limit: u32,
    ) -> PortResult<Vec<TeamOption>> {
        self.store
            .list_team_options(search, limit)
            .await
            .map_err(map_persistence_error)
    }
    async fn list_teams(&self, query: &TeamListQuery) -> PortResult<TeamListPage> {
        self.store
            .list_teams(query)
            .await
            .map_err(map_persistence_error)
    }
    async fn read_team(&self, team_id: Uuid) -> PortResult<TeamDetail> {
        self.store
            .read_team(team_id)
            .await
            .map_err(map_persistence_error)
    }
    async fn update_team(&self, team_id: Uuid, draft: &TeamDraft) -> PortResult<TeamRecord> {
        self.store
            .update_team(team_id, draft)
            .await
            .map_err(map_persistence_error)
    }
    async fn add_team_name(&self, draft: &TeamNameDraft) -> PortResult<TeamNameRecord> {
        self.store
            .add_team_name(draft)
            .await
            .map_err(map_persistence_error)
    }
    async fn upsert_team_profile(
        &self,
        team_id: Uuid,
        draft: &TeamProfileDraft,
    ) -> PortResult<TeamProfileRecord> {
        self.store
            .upsert_team_profile(team_id, draft)
            .await
            .map_err(map_persistence_error)
    }
}
#[async_trait]
impl TeamLifecyclePort for ActiveDatabase {
    async fn bulk_delete_teams(&self, team_ids: &[Uuid]) -> PortResult<BulkDeleteResult> {
        self.store
            .bulk_delete_teams(team_ids)
            .await
            .map_err(map_persistence_error)
    }
    async fn preview_force_delete_team(&self, team_id: Uuid) -> PortResult<TeamForceDeletePreview> {
        self.store
            .preview_force_delete_team(team_id)
            .await
            .map_err(map_persistence_error)
    }
    async fn force_delete_team(
        &self,
        request: &TeamForceDeleteRequest,
    ) -> PortResult<TeamForceDeleteResult> {
        self.store
            .force_delete_team(request)
            .await
            .map_err(map_persistence_error)
    }
}
#[async_trait]
impl PlayerCatalogPort for ActiveDatabase {
    async fn reference_data(&self) -> PortResult<PlayerCatalogReferenceData> {
        self.store
            .player_catalog_reference_data()
            .await
            .map_err(map_persistence_error)
    }
    async fn create_player(&self, draft: &PlayerDraft) -> PortResult<PlayerRecord> {
        self.store
            .create_player(draft)
            .await
            .map_err(map_persistence_error)
    }
    async fn update_player(
        &self,
        player_id: Uuid,
        draft: &PlayerDraft,
    ) -> PortResult<PlayerRecord> {
        self.store
            .update_player(player_id, draft)
            .await
            .map_err(map_persistence_error)
    }
    async fn delete_player(&self, player_id: Uuid) -> PortResult<()> {
        self.store
            .delete_player(player_id)
            .await
            .map_err(map_persistence_error)
    }
    async fn bulk_delete_players(&self, player_ids: &[Uuid]) -> PortResult<BulkDeleteResult> {
        self.store
            .bulk_delete_players(player_ids)
            .await
            .map_err(map_persistence_error)
    }
    async fn list_players(&self, query: &PlayerListQuery) -> PortResult<PlayerListPage> {
        self.store
            .list_players(query)
            .await
            .map_err(map_persistence_error)
    }
    async fn read_player(&self, player_id: Uuid) -> PortResult<PlayerDetail> {
        self.store
            .read_player(player_id)
            .await
            .map_err(map_persistence_error)
    }
    async fn add_player_name(&self, draft: &PlayerNameDraft) -> PortResult<PlayerNameRecord> {
        self.store
            .add_player_name(draft)
            .await
            .map_err(map_persistence_error)
    }
    async fn assign_player_position(
        &self,
        draft: &PlayerPositionDraft,
    ) -> PortResult<PlayerPositionRecord> {
        self.store
            .assign_player_position(draft)
            .await
            .map_err(map_persistence_error)
    }
    async fn add_player_team_period(
        &self,
        draft: &PlayerTeamPeriodDraft,
    ) -> PortResult<PlayerTeamPeriodRecord> {
        self.store
            .add_player_team_period(draft)
            .await
            .map_err(map_persistence_error)
    }
}
#[async_trait]
impl PlayerSignalPort for ActiveDatabase {
    async fn add_availability(
        &self,
        draft: &PlayerAvailabilityDraft,
    ) -> PortResult<PlayerAvailabilityRecord> {
        self.store
            .add_player_availability(draft)
            .await
            .map_err(map_persistence_error)
    }
    async fn add_ability_observation(
        &self,
        draft: &PlayerAbilityObservationDraft,
    ) -> PortResult<PlayerAbilityObservationRecord> {
        self.store
            .add_player_ability_observation(draft)
            .await
            .map_err(map_persistence_error)
    }
    async fn add_dynamic_tag(
        &self,
        draft: &PlayerDynamicTagDraft,
    ) -> PortResult<PlayerDynamicTagRecord> {
        self.store
            .add_player_dynamic_tag(draft)
            .await
            .map_err(map_persistence_error)
    }
    async fn calculate_match_contribution(
        &self,
        request: &PlayerMatchContributionRequest,
    ) -> PortResult<PlayerMatchContribution> {
        self.store
            .calculate_player_match_contribution(request)
            .await
            .map_err(map_persistence_error)
    }
}
#[async_trait]
impl CoachCatalogPort for ActiveDatabase {
    async fn create_coach(&self, draft: &CoachDraft) -> PortResult<CoachRecord> {
        self.store
            .create_coach(draft)
            .await
            .map_err(map_persistence_error)
    }
    async fn list_coaches(&self, query: &CoachListQuery) -> PortResult<Vec<CoachListItem>> {
        self.store
            .list_coaches(query)
            .await
            .map_err(map_persistence_error)
    }
    async fn read_coach(&self, coach_id: Uuid) -> PortResult<CoachDetail> {
        self.store
            .read_coach(coach_id)
            .await
            .map_err(map_persistence_error)
    }
    async fn add_coach_name(&self, draft: &CoachNameDraft) -> PortResult<CoachNameRecord> {
        self.store
            .add_coach_name(draft)
            .await
            .map_err(map_persistence_error)
    }
    async fn add_team_coach_period(
        &self,
        draft: &TeamCoachPeriodDraft,
    ) -> PortResult<TeamCoachPeriodRecord> {
        self.store
            .add_team_coach_period(draft)
            .await
            .map_err(map_persistence_error)
    }
}
#[async_trait]
impl EntityReferencePort for ActiveDatabase {
    async fn list_references(
        &self,
        query: &EntityReferenceQuery,
    ) -> PortResult<Vec<EntityReferenceRecord>> {
        self.store
            .list_entity_references(query)
            .await
            .map_err(map_persistence_error)
    }
    async fn resolve_reference(
        &self,
        request: &EntityMatchRequest,
    ) -> PortResult<EntityMatchResult> {
        self.store
            .resolve_entity_reference(request)
            .await
            .map_err(map_persistence_error)
    }
    async fn check_deletion(
        &self,
        entity_type: &str,
        entity_id: Uuid,
    ) -> PortResult<EntityDeletionCheck> {
        self.store
            .check_entity_deletion(entity_type, entity_id)
            .await
            .map_err(map_persistence_error)
    }
    async fn bulk_archive(
        &self,
        entity_type: &str,
        entity_ids: &[Uuid],
    ) -> PortResult<BulkArchiveResult> {
        self.store
            .bulk_archive_entities(entity_type, entity_ids)
            .await
            .map_err(map_persistence_error)
    }
    async fn create_data_provider(
        &self,
        draft: &DataProviderDraft,
    ) -> PortResult<DataProviderRecord> {
        self.store
            .create_data_provider(draft)
            .await
            .map_err(map_persistence_error)
    }
    async fn add_external_id(
        &self,
        draft: &ExternalEntityIdDraft,
    ) -> PortResult<ExternalEntityIdRecord> {
        self.store
            .add_external_entity_id(draft)
            .await
            .map_err(map_persistence_error)
    }
}
