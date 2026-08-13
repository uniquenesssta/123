use super::super::port_registry::{DatabaseHealth, DatabaseStats, PersistenceStore};
use super::map_persistence_error;
use crate::ports::{
    database::{
        DatabaseHealthSnapshot, DatabaseLifecyclePort, DatabaseObservabilityPort,
        DatabaseStatistics,
    },
    PortResult,
};
use async_trait::async_trait;

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

#[async_trait]
impl DatabaseLifecyclePort for PersistenceStore {
    async fn migrate(&self) -> PortResult<()> {
        PersistenceStore::migrate(self)
            .await
            .map_err(map_persistence_error)
    }

    async fn recover_interrupted_work(&self) -> PortResult<()> {
        PersistenceStore::recover_interrupted_jobs(self)
            .await
            .map_err(map_persistence_error)?;
        PersistenceStore::recover_interrupted_api_workspace_operations(self)
            .await
            .map_err(map_persistence_error)?;
        Ok(())
    }

    async fn reset_to_pristine(&self) -> PortResult<()> {
        PersistenceStore::reset_to_pristine(self)
            .await
            .map_err(map_persistence_error)
    }

    async fn close(&self) -> PortResult<()> {
        PersistenceStore::close(self).await;
        Ok(())
    }
}

#[async_trait]
impl DatabaseObservabilityPort for PersistenceStore {
    async fn health(&self) -> PortResult<DatabaseHealthSnapshot> {
        let health = PersistenceStore::health(self)
            .await
            .map_err(map_persistence_error)?;
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
        let statistics = PersistenceStore::stats(self)
            .await
            .map_err(map_persistence_error)?;
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
