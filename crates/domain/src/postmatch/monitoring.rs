use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

pub const POSTMATCH_MONITORING_VERSION: &str = "postmatch-monitoring-v1";
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostmatchDriftFindingRecord {
    pub metric_name: String,
    pub baseline_mean: f64,
    pub current_mean: f64,
    pub absolute_delta: f64,
    pub relative_delta: Option<f64>,
    pub severity: String,
    pub direction: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostmatchDriftRunRecord {
    pub id: Uuid,
    pub competition_id: Uuid,
    pub competition_name: String,
    pub competition_profile_id: Uuid,
    pub model_version_id: Uuid,
    pub model_version: String,
    pub parameter_set_id: Uuid,
    pub parameter_version: String,
    pub horizon: String,
    pub partition_key: String,
    pub baseline_size: u64,
    pub current_size: u64,
    pub baseline_window: Value,
    pub current_window: Value,
    pub status: String,
    pub run_key: String,
    pub calculation_version: String,
    pub findings: Vec<PostmatchDriftFindingRecord>,
    pub generated_at: DateTime<Utc>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostmatchMonitoringRequest {
    pub competition_id: Uuid,
    pub horizon: String,
    #[serde(default = "default_postmatch_baseline_size")]
    pub baseline_size: usize,
    #[serde(default = "default_postmatch_current_size")]
    pub current_size: usize,
}
fn default_postmatch_baseline_size() -> usize {
    100
}
fn default_postmatch_current_size() -> usize {
    50
}
