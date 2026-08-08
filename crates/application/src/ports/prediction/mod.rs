use crate::ports::PortResult;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use football_domain::{
    P4FreezeReadiness, P4FreezeTaskDraft, P4FreezeTaskEventRecord, P4FreezeTaskRecord,
    P4FreezeTaskTransition, PredictionSummary, PreparedMatchPredictionInput, RouteDecision,
};
use football_model_api::{ModelOutput, ModelRequest};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct ModelRunHistoryItem {
    pub id: Uuid,
    pub match_key: String,
    pub competition_name: Option<String>,
    pub home_team_name: Option<String>,
    pub away_team_name: Option<String>,
    pub kickoff_time: Option<DateTime<Utc>>,
    pub snapshot_type: String,
    pub model_key: String,
    pub model_version: String,
    pub parameter_version: String,
    pub rule_package_name: Option<String>,
    pub summary: PredictionSummary,
    pub top_scoreline: Option<String>,
    pub top_scoreline_probability: Option<f64>,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub duration_ms: Option<i64>,
    pub input_readiness_level: String,
    pub input_readiness_score: Option<i16>,
    pub input_manifest_sha256: String,
}

#[derive(Debug, Clone)]
pub struct SerializedModelRun {
    pub json: String,
}

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
    async fn list_recent_runs(&self, limit: i64) -> PortResult<Vec<ModelRunHistoryItem>>;
    async fn read_run_document(&self, run_id: Uuid) -> PortResult<SerializedModelRun>;
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
