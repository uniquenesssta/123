use crate::ports::PortResult;
use async_trait::async_trait;
use football_domain::{
    ReleaseAcceptanceRequest, ReleaseAcceptanceRun, ReleaseAcceptanceRunSummary,
    ReleaseAcceptanceRuntimeFacts,
};
use uuid::Uuid;

#[async_trait]
pub trait ReleaseAcceptancePort: Send + Sync {
    async fn runtime_facts(&self) -> PortResult<ReleaseAcceptanceRuntimeFacts>;
    async fn persist_run(
        &self,
        request: &ReleaseAcceptanceRequest,
    ) -> PortResult<ReleaseAcceptanceRun>;
    async fn list_runs(&self, limit: i64) -> PortResult<Vec<ReleaseAcceptanceRunSummary>>;
    async fn read_run(&self, run_id: Uuid) -> PortResult<ReleaseAcceptanceRun>;
}
