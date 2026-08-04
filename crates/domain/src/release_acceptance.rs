use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

pub const RELEASE_ACCEPTANCE_CONTRACT_VERSION: &str = "football.integration-j.acceptance.v1";
pub const RELEASE_ACCEPTANCE_FIXTURE_VERSION: &str = "p4-fixed-fixture-v1";

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ReleaseAcceptanceStatus {
    Pass,
    Warning,
    Blocked,
}

impl ReleaseAcceptanceStatus {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Pass => "pass",
            Self::Warning => "warning",
            Self::Blocked => "blocked",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReleaseAcceptanceRequest {
    #[serde(default = "default_performance_window_days")]
    pub performance_window_days: u32,
    #[serde(default = "default_cost_window_days")]
    pub cost_window_days: u32,
    #[serde(default)]
    pub daily_cost_budget_usd: Option<f64>,
    #[serde(default)]
    pub monthly_cost_budget_usd: Option<f64>,
    #[serde(default)]
    pub requested_by: Option<String>,
}

fn default_performance_window_days() -> u32 { 30 }
fn default_cost_window_days() -> u32 { 30 }

impl Default for ReleaseAcceptanceRequest {
    fn default() -> Self {
        Self {
            performance_window_days: default_performance_window_days(),
            cost_window_days: default_cost_window_days(),
            daily_cost_budget_usd: None,
            monthly_cost_budget_usd: None,
            requested_by: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReleaseAcceptanceCheck {
    pub id: Uuid,
    pub run_id: Uuid,
    pub sequence_no: i32,
    pub category: String,
    pub check_code: String,
    pub title: String,
    pub status: ReleaseAcceptanceStatus,
    pub summary: String,
    pub remediation: Option<String>,
    pub evidence: Value,
    pub duration_ms: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ReleaseAcceptanceCategorySummary {
    pub category: String,
    pub passed: u32,
    pub warnings: u32,
    pub blocked: u32,
}

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReleaseAcceptanceRun {
    pub id: Uuid,
    pub app_version: String,
    pub contract_version: String,
    pub fixture_version: String,
    pub overall_status: ReleaseAcceptanceStatus,
    pub started_at: DateTime<Utc>,
    pub completed_at: DateTime<Utc>,
    pub requested_by: Option<String>,
    pub report_sha256: String,
    pub passed_count: u32,
    pub warning_count: u32,
    pub blocked_count: u32,
    pub category_summaries: Vec<ReleaseAcceptanceCategorySummary>,
    pub performance: ReleaseAcceptancePerformanceSummary,
    pub cost: ReleaseAcceptanceCostSummary,
    pub checks: Vec<ReleaseAcceptanceCheck>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReleaseAcceptanceRunSummary {
    pub id: Uuid,
    pub app_version: String,
    pub overall_status: ReleaseAcceptanceStatus,
    pub completed_at: DateTime<Utc>,
    pub requested_by: Option<String>,
    pub passed_count: u32,
    pub warning_count: u32,
    pub blocked_count: u32,
    pub report_sha256: String,
}

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
