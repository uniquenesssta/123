use crate::ports::PortResult;
use async_trait::async_trait;
use football_domain::{
    ReleaseAcceptanceRun, ReleaseAcceptanceRunSummary, ReleaseAcceptanceRuntimeFacts,
};
use uuid::Uuid;

#[async_trait]
pub trait ReleaseAcceptancePort: Send + Sync {
    async fn runtime_facts(
        &self,
        performance_window_days: u32,
        cost_window_days: u32,
    ) -> PortResult<ReleaseAcceptanceRuntimeFacts>;
    async fn persist_run(&self, run: &ReleaseAcceptanceRun) -> PortResult<()>;
    async fn list_runs(&self, limit: u32) -> PortResult<Vec<ReleaseAcceptanceRunSummary>>;
    async fn read_run(&self, run_id: Uuid) -> PortResult<ReleaseAcceptanceRun>;
}
