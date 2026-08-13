use super::super::port_registry::PersistenceStore;
use super::map_persistence_error;
use crate::ports::{release::ReleaseAcceptancePort, PortResult};
use async_trait::async_trait;
use football_domain::{
    ReleaseAcceptanceRun, ReleaseAcceptanceRunSummary, ReleaseAcceptanceRuntimeFacts,
};
use uuid::Uuid;

#[async_trait]
impl ReleaseAcceptancePort for PersistenceStore {
    async fn runtime_facts(
        &self,
        performance_window_days: u32,
        cost_window_days: u32,
    ) -> PortResult<ReleaseAcceptanceRuntimeFacts> {
        self.release_acceptance_runtime_facts(performance_window_days, cost_window_days)
            .await
            .map_err(map_persistence_error)
    }

    async fn persist_run(&self, run: &ReleaseAcceptanceRun) -> PortResult<()> {
        self.persist_release_acceptance_run(run)
            .await
            .map_err(map_persistence_error)
    }

    async fn list_runs(&self, limit: u32) -> PortResult<Vec<ReleaseAcceptanceRunSummary>> {
        self.list_release_acceptance_runs(limit)
            .await
            .map_err(map_persistence_error)
    }

    async fn read_run(&self, run_id: Uuid) -> PortResult<ReleaseAcceptanceRun> {
        self.read_release_acceptance_run(run_id)
            .await
            .map_err(map_persistence_error)
    }
}
