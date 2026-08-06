use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelIdentity {
    pub model_id: String,
    pub model_version: String,
    pub parameter_version: String,
    pub rule_package_version: Option<String>,
}
