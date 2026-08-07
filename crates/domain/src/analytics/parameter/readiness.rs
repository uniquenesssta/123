use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

use super::tuning::default_tuning_minimum_samples;

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
