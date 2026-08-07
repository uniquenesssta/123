use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

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

fn default_parameter_lifecycle_snapshot_type() -> String {
    "T-1h".to_string()
}

fn default_tuning_max_change() -> f64 {
    0.05
}

pub(super) fn default_tuning_minimum_samples() -> u64 {
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
