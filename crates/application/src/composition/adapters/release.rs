use super::super::port_registry::{map_persistence_error, ActiveDatabase};
use crate::ports::{release::ReleaseAcceptancePort, PortResult};
use async_trait::async_trait;
use football_domain::{
    ReleaseAcceptanceRun, ReleaseAcceptanceRunSummary, ReleaseAcceptanceRuntimeFacts,
};
use uuid::Uuid;

#[async_trait]
impl ReleaseAcceptancePort for ActiveDatabase {
    async fn runtime_facts(
        &self,
        performance_window_days: u32,
        cost_window_days: u32,
    ) -> PortResult<ReleaseAcceptanceRuntimeFacts> {
        self.transition_store()
            .release_acceptance_runtime_facts(performance_window_days, cost_window_days)
            .await
            .map_err(map_persistence_error)
    }

    async fn persist_run(&self, run: &ReleaseAcceptanceRun) -> PortResult<()> {
        self.transition_store()
            .persist_release_acceptance_run(run)
            .await
            .map_err(map_persistence_error)
    }

    async fn list_runs(&self, limit: u32) -> PortResult<Vec<ReleaseAcceptanceRunSummary>> {
        self.transition_store()
            .list_release_acceptance_runs(limit)
            .await
            .map_err(map_persistence_error)
    }

    async fn read_run(&self, run_id: Uuid) -> PortResult<ReleaseAcceptanceRun> {
        self.transition_store()
            .read_release_acceptance_run(run_id)
            .await
            .map_err(map_persistence_error)
    }
}
