use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

pub const ANALYTICS_CALCULATION_VERSION: &str = "phase5-analytics-v1";
pub const AI_ANALYSIS_PACKAGE_FORMAT: &str = "football.ai-analysis-package.v1";
pub const AI_ANALYSIS_RESPONSE_FORMAT: &str = "football.ai-analysis-response.v1";

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
pub struct DataQualityFinding {
    pub id: Uuid,
    pub scan_id: Uuid,
    pub severity: String,
    pub category: String,
    pub finding_code: String,
    pub entity_type: String,
    pub entity_id: Option<String>,
    pub message: String,
    pub evidence: Value,
    pub status: String,
    pub detected_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataQualitySummary {
    pub scan_id: Option<Uuid>,
    pub generated_at: Option<DateTime<Utc>>,
    pub critical: i64,
    pub warning: i64,
    pub info: i64,
    pub open_total: i64,
    pub findings: Vec<DataQualityFinding>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryPerformanceFinding {
    pub schema_name: String,
    pub table_name: String,
    pub estimated_rows: i64,
    pub table_size_bytes: i64,
    pub sequential_scans: i64,
    pub index_scans: i64,
    pub dead_rows: i64,
    pub last_analyze: Option<DateTime<Utc>>,
    pub severity: String,
    pub recommendation: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryPerformanceSummary {
    pub captured_at: Option<DateTime<Utc>>,
    pub database_size_bytes: i64,
    pub tables: Vec<QueryPerformanceFinding>,
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

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum JobStatus {
    Queued,
    Running,
    Succeeded,
    Failed,
    Cancelled,
}

impl JobStatus {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Succeeded => "succeeded",
            Self::Failed => "failed",
            Self::Cancelled => "cancelled",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackgroundJob {
    pub id: Uuid,
    pub job_type: String,
    pub status: JobStatus,
    pub progress: f64,
    pub payload: Value,
    pub result: Option<Value>,
    pub error_message: Option<String>,
    pub priority: i32,
    pub attempts: i32,
    pub max_attempts: i32,
    pub cancellation_requested: bool,
    pub available_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub finished_at: Option<DateTime<Utc>>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnqueueJobDraft {
    pub job_type: String,
    #[serde(default)]
    pub payload: Value,
    #[serde(default)]
    pub idempotency_key: Option<String>,
    #[serde(default)]
    pub available_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub priority: i32,
    #[serde(default = "default_max_attempts")]
    pub max_attempts: i32,
}

fn default_max_attempts() -> i32 {
    3
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiAnalysisPackageManifest {
    pub format_version: String,
    pub package_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub calculation_version: String,
    pub sample_size: u64,
    pub content_sha256: String,
    pub files: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiAnalysisPackageData {
    pub overview: AnalyticsOverview,
    pub database_summary: Value,
    pub player_review_summary: Value,
    pub team_review_summary: Value,
    pub ability_candidates: Value,
    pub schema_summary: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiAnalysisPackageSummary {
    pub package_id: Uuid,
    pub output_path: String,
    pub content_sha256: String,
    pub sample_size: u64,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AiAnalysisSuggestionDraft {
    pub suggestion_type: String,
    pub title: String,
    pub summary: String,
    #[serde(default)]
    pub severity: String,
    #[serde(default)]
    pub scope: Value,
    #[serde(default)]
    pub payload: Value,
    #[serde(default)]
    pub evidence: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AiAnalysisResponseManifest {
    pub format_version: String,
    pub response_id: Uuid,
    pub created_at: DateTime<Utc>,
    #[serde(default)]
    pub source_package_id: Option<Uuid>,
    pub content_sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiAnalysisResponsePreview {
    pub manifest: AiAnalysisResponseManifest,
    pub suggestions: Vec<AiAnalysisSuggestionDraft>,
    pub blocking_errors: Vec<String>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiAnalysisSuggestionRecord {
    pub id: Uuid,
    pub response_id: Uuid,
    pub suggestion_type: String,
    pub title: String,
    pub summary: String,
    pub severity: String,
    pub scope: Value,
    pub payload: Value,
    pub evidence: Value,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub decided_at: Option<DateTime<Utc>>,
    pub decision_note: Option<String>,
    pub linked_candidate_id: Option<Uuid>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AiSuggestionDecision {
    Accept,
    Reject,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiSuggestionDecisionDraft {
    pub suggestion_id: Uuid,
    pub decision: AiSuggestionDecision,
    #[serde(default)]
    pub decided_by: Option<String>,
    #[serde(default)]
    pub decision_note: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DataQualityDecision {
    Resolve,
    Ignore,
}

impl DataQualityDecision {
    pub const fn as_status(self) -> &'static str {
        match self {
            Self::Resolve => "resolved",
            Self::Ignore => "ignored",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataQualityDecisionDraft {
    pub finding_id: Uuid,
    pub decision: DataQualityDecision,
    #[serde(default)]
    pub resolution_note: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterTuningDraft {
    #[serde(default)]
    pub competition_id: Option<Uuid>,
    #[serde(default = "default_parameter_lifecycle_snapshot_type")]
    pub snapshot_type: String,
    pub target_module: String,
    #[serde(default = "default_tuning_max_change")]
    pub max_relative_change: f64,
    #[serde(default = "default_tuning_minimum_samples")]
    pub minimum_sample_size: u64,
}

fn default_tuning_max_change() -> f64 {
    0.05
}
fn default_tuning_minimum_samples() -> u64 {
    50
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ParameterTuningDecision {
    AcceptForBacktest,
    Reject,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterTuningDecisionDraft {
    pub candidate_id: Uuid,
    pub decision: ParameterTuningDecision,
    #[serde(default)]
    pub decision_note: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterTuningCandidateRecord {
    pub id: Uuid,
    pub competition_id: Option<Uuid>,
    pub competition_name: Option<String>,
    pub competition_profile_id: Option<Uuid>,
    pub partition_key: Option<String>,
    pub model_key: String,
    pub model_version: String,
    pub parameter_version: String,
    pub snapshot_type: String,
    pub target_module: String,
    pub sample_size: u64,
    pub baseline_model_version_id: Option<Uuid>,
    pub baseline_parameter_set_id: Option<Uuid>,
    pub candidate_model_version_id: Option<Uuid>,
    pub candidate_parameter_set_id: Option<Uuid>,
    pub candidate_model_version: Option<String>,
    pub candidate_parameter_version: Option<String>,
    pub candidate_definition_sha256: Option<String>,
    pub baseline_metrics: Value,
    pub calibration_bias: Value,
    pub proposed_adjustments: Value,
    pub constraints: Value,
    pub training_window: Value,
    pub validation_window: Value,
    pub holdout_window: Value,
    pub rationale: String,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub decided_at: Option<DateTime<Utc>>,
    pub decision_note: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterLifecycleReadinessRequest {
    pub competition_id: Option<Uuid>,
    #[serde(default = "default_parameter_lifecycle_snapshot_type")]
    pub snapshot_type: String,
    #[serde(default = "default_tuning_minimum_samples")]
    pub minimum_sample_size: u64,
}

fn default_parameter_lifecycle_snapshot_type() -> String {
    "T-1h".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterLifecycleReadiness {
    pub partition_key: String,
    pub competition_id: Option<Uuid>,
    pub competition_name: Option<String>,
    pub competition_profile_id: Option<Uuid>,
    pub snapshot_type: String,
    pub h_contract_ready: bool,
    pub h_contract_version: Option<String>,
    pub settled_sample_count: u64,
    pub eligible_sample_count: u64,
    pub minimum_sample_size: u64,
    pub active_model_version_id: Option<Uuid>,
    pub active_parameter_set_id: Option<Uuid>,
    pub active_model_version: Option<String>,
    pub active_parameter_version: Option<String>,
    pub blocked_reasons: Vec<String>,
    pub ready_for_shadow_validation: bool,
    pub ready_for_promotion: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterCandidateBaseline {
    pub competition_id: Uuid,
    pub competition_name: String,
    pub competition_profile_id: Uuid,
    pub binding_id: Uuid,
    pub rule_package_id: Uuid,
    pub rule_package_version: String,
    pub model_key: String,
    pub model_version_id: Uuid,
    pub model_version: String,
    pub engine_version: String,
    pub input_schema_version: String,
    pub output_schema_version: String,
    pub parameter_set_id: Uuid,
    pub parameter_version: String,
    pub parameters: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterCandidateArtifactDraft {
    pub candidate_id: Uuid,
    pub baseline: ParameterCandidateBaseline,
    pub candidate_model_version_id: Uuid,
    pub candidate_parameter_set_id: Uuid,
    pub candidate_model_version: String,
    pub candidate_parameter_version: String,
    pub candidate_parameters: Value,
    pub candidate_definition_sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterReplayFixture {
    pub review_id: Uuid,
    pub run_id: Uuid,
    pub match_id: Uuid,
    pub match_key: String,
    pub competition_id: Option<Uuid>,
    pub season_id: Option<Uuid>,
    pub stage_id: Option<Uuid>,
    pub competition_kind: String,
    pub competition_profile_id: Uuid,
    pub kickoff_time: DateTime<Utc>,
    pub home_team_name: String,
    pub away_team_name: String,
    pub snapshot_type: String,
    pub input_payload: Value,
    pub rule_package_version: Option<String>,
    pub actual_home_goals: i16,
    pub actual_away_goals: i16,
    pub baseline_home_win: f64,
    pub baseline_draw: f64,
    pub baseline_away_win: f64,
    pub baseline_scoreline_probability: Option<f64>,
    pub data_coverage: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterShadowValidationRequest {
    pub candidate_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterShadowValidationRecord {
    pub id: Uuid,
    pub candidate_id: Uuid,
    pub validation_key: String,
    pub partition_key: String,
    pub sample_count: u64,
    pub baseline_metrics: Value,
    pub candidate_metrics: Value,
    pub metric_deltas: Value,
    pub gate_results: Value,
    pub status: String,
    pub generated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterPromotionRequest {
    pub candidate_id: Uuid,
    #[serde(default)]
    pub decided_by: Option<String>,
    pub decision_note: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterRollbackRequest {
    pub candidate_id: Uuid,
    #[serde(default)]
    pub decided_by: Option<String>,
    pub decision_note: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterPromotionDecisionRecord {
    pub id: Uuid,
    pub candidate_id: Uuid,
    pub decision: String,
    pub previous_binding_state: Value,
    pub new_binding_state: Value,
    pub decided_by: Option<String>,
    pub decision_note: String,
    pub created_at: DateTime<Utc>,
}
