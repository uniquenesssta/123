use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataProviderDraft {
    pub code: String,
    pub name: String,
    pub provider_type: String,
    #[serde(default)]
    pub base_url: Option<String>,
    #[serde(default)]
    pub metadata: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataProviderRecord {
    pub id: Uuid,
    pub code: String,
    pub name: String,
    pub provider_type: String,
    pub base_url: Option<String>,
    pub is_active: bool,
}
