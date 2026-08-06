use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleRouting {
    pub model_id: String,
    pub model_version: String,
    pub parameter_version: String,
    #[serde(default)]
    pub priority: i32,
    #[serde(default)]
    pub activate_as_type_default: bool,
    #[serde(default)]
    pub supported_snapshot_types: Vec<String>,
}
