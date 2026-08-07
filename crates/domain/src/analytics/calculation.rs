use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::quality::DataQualitySummary;
use super::query_performance::QueryPerformanceSummary;

pub const ANALYTICS_CALCULATION_VERSION: &str = "phase5-analytics-v1";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsRefreshRequest {
    #[serde(default)]
    pub competition_id: Option<Uuid>,
    #[serde(default)]
    pub window_start: Option<DateTime<Utc>>,
    #[serde(default)]
    pub window_end: Option<DateTime<Utc>>,
    #[serde(default = "default_bucket_count")]
    pub bucket_count: u8,
    #[serde(default = "default_baseline_size")]
    pub baseline_size: usize,
    #[serde(default = "default_current_size")]
    pub current_size: usize,
}

fn default_bucket_count() -> u8 {
    10
}

fn default_baseline_size() -> usize {
    100
}

fn default_current_size() -> usize {
    50
}

impl Default for AnalyticsRefreshRequest {
    fn default() -> Self {
        Self {
            competition_id: None,
            window_start: None,
            window_end: None,
            bucket_count: default_bucket_count(),
            baseline_size: default_baseline_size(),
            current_size: default_current_size(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluationSample {
    pub review_id: Uuid,
    pub run_id: Uuid,
    pub model_version_id: Uuid,
    pub parameter_set_id: Uuid,
    pub model_key: String,
    pub model_version: String,
    pub parameter_version: String,
    pub competition_id: Option<Uuid>,
    pub competition_name: Option<String>,
    pub season_id: Option<Uuid>,
    pub stage_id: Option<Uuid>,
    pub snapshot_type: String,
    pub kickoff_time: DateTime<Utc>,
    pub actual_outcome: String,
    pub home_win: f64,
    pub draw: f64,
    pub away_win: f64,
    pub log_loss: f64,
    pub brier: f64,
    pub scoreline_nll: Option<f64>,
    pub data_coverage: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalibrationBucket {
    pub outcome: String,
    pub bucket_index: u8,
    pub lower_bound: f64,
    pub upper_bound: f64,
    pub sample_size: u64,
    pub predicted_mean: f64,
    pub actual_rate: f64,
    pub absolute_gap: f64,
    pub ece_component: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelComparisonRow {
    pub model_key: String,
    pub model_version: String,
    pub parameter_version: String,
    pub snapshot_type: String,
    pub sample_size: u64,
    pub average_log_loss: f64,
    pub average_brier: f64,
    pub average_scoreline_nll: Option<f64>,
    pub average_data_coverage: f64,
    pub rank: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriftFinding {
    pub metric_name: String,
    pub baseline_mean: f64,
    pub current_mean: f64,
    pub absolute_delta: f64,
    pub relative_delta: Option<f64>,
    pub baseline_size: u64,
    pub current_size: u64,
    pub severity: String,
    pub direction: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsCalculation {
    pub calculation_version: String,
    pub generated_at: DateTime<Utc>,
    pub sample_size: u64,
    pub average_log_loss: Option<f64>,
    pub average_brier: Option<f64>,
    pub average_scoreline_nll: Option<f64>,
    pub expected_calibration_error: Option<f64>,
    pub calibration: Vec<CalibrationBucket>,
    pub comparisons: Vec<ModelComparisonRow>,
    pub drift: Vec<DriftFinding>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsOverview {
    pub generated_at: Option<DateTime<Utc>>,
    pub calculation_version: String,
    pub sample_size: u64,
    pub average_log_loss: Option<f64>,
    pub average_brier: Option<f64>,
    pub average_scoreline_nll: Option<f64>,
    pub expected_calibration_error: Option<f64>,
    pub comparisons: Vec<ModelComparisonRow>,
    pub calibration: Vec<CalibrationBucket>,
    pub drift: Vec<DriftFinding>,
    pub data_quality: DataQualitySummary,
    pub query_performance: QueryPerformanceSummary,
}
