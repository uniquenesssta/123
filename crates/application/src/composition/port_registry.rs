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
    rules::{RulePackagePort, RuleRoutingPort},
    PortError, PortErrorKind, PortResult,
};
use async_trait::async_trait;
use football_domain::{
    CompetitionBindingDraft, CompetitionBindingSummary, CompetitionDraft, CompetitionKind,
    CompetitionRecord, RoundDraft, RoundRecord, RouteDecision, RouteRequest, RulePackageDraft,
    RulePackageSummary, SeasonDraft, SeasonRecord, StageDraft, StageRecord,
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
