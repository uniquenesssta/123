use crate::ports::PortResult;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use football_domain::{
    P4FreezeReadiness, P4FreezeTaskDraft, P4FreezeTaskEventRecord, P4FreezeTaskRecord,
    P4FreezeTaskTransition, PreparedMatchPredictionInput, RouteDecision,
};
use football_model_api::{ModelOutput, ModelRequest};
use uuid::Uuid;

#[async_trait]
pub trait PredictionInputPort: Send + Sync {
    async fn prepare_match_input(
        &self,
        match_id: Uuid,
        snapshot_type: &str,
        model_family: &str,
    ) -> PortResult<PreparedMatchPredictionInput>;
    async fn prepare_match_input_at(
        &self,
        match_id: Uuid,
        snapshot_type: &str,
        model_family: &str,
        reference_time: DateTime<Utc>,
    ) -> PortResult<PreparedMatchPredictionInput>;
}

#[async_trait]
pub trait ModelRunPort: Send + Sync {
    async fn save_successful_run(
        &self,
        decision: &RouteDecision,
        request: &ModelRequest,
        output: &ModelOutput,
        duration_ms: i64,
    ) -> PortResult<Uuid>;
    async fn hide_run_from_history(&self, run_id: Uuid, reason: Option<&str>) -> PortResult<()>;
}

#[async_trait]
pub trait PredictionWorkflowPort: Send + Sync {
    async fn create_freeze_task(&self, draft: &P4FreezeTaskDraft)
        -> PortResult<P4FreezeTaskRecord>;
    async fn read_freeze_task(&self, task_id: Uuid) -> PortResult<P4FreezeTaskRecord>;
    async fn list_freeze_task_events(
        &self,
        task_id: Uuid,
    ) -> PortResult<Vec<P4FreezeTaskEventRecord>>;
    async fn transition_freeze_task(
        &self,
        task_id: Uuid,
        transition: &P4FreezeTaskTransition,
    ) -> PortResult<P4FreezeTaskRecord>;
    async fn freeze_readiness(&self, task_id: Uuid) -> PortResult<P4FreezeReadiness>;
}
