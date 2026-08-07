use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::check::{ReleaseAcceptanceCategorySummary, ReleaseAcceptanceCheck};
use super::contract::ReleaseAcceptanceStatus;
use super::metrics::{ReleaseAcceptanceCostSummary, ReleaseAcceptancePerformanceSummary};

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
