use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ReleaseAcceptancePerformanceSummary {
    pub database_latency_ms: u64,
    pub recent_model_run_count: u64,
    pub recent_model_run_p95_ms: Option<f64>,
    pub recent_model_failure_count: u64,
    pub query_warning_count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ReleaseAcceptanceCostSummary {
    pub window_days: u32,
    pub completed_requests: u64,
    pub failed_requests: u64,
    pub search_calls: u64,
    pub estimated_cost_usd: f64,
    pub latest_day_cost_usd: f64,
    pub daily_budget_usd: Option<f64>,
    pub monthly_budget_usd: Option<f64>,
}
