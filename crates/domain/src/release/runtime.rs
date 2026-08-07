use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ReleaseAcceptanceRuntimeFacts {
    pub migration_count: i64,
    pub database_latency_ms: u128,
    pub integration_stages: Vec<String>,
    pub immutable_trigger_count: i64,
    pub provider_boundary_artifact_count: i64,
    pub freeze_task_count: i64,
    pub frozen_snapshot_count: i64,
    pub settlement_count: i64,
    pub evidence_decision_count: i64,
    pub shadow_validation_count: i64,
    pub promotion_decision_count: i64,
    pub recent_model_run_count: i64,
    pub recent_model_run_p95_ms: Option<f64>,
    pub recent_model_failure_count: i64,
    pub query_warning_count: i64,
    pub completed_requests: i64,
    pub failed_requests: i64,
    pub search_calls: i64,
    pub estimated_cost_usd: f64,
    pub latest_day_cost_usd: f64,
}
