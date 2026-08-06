use super::default_team_type;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamDraft {
    pub canonical_name: String,
    #[serde(default)]
    pub country_code: Option<String>,
    #[serde(default)]
    pub metadata: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamRecord {
    pub id: Uuid,
    pub canonical_name: String,
    pub normalized_name: String,
    pub country_code: Option<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamOption {
    pub id: Uuid,
    pub canonical_name: String,
    pub country_code: Option<String>,
    #[serde(default = "default_team_type")]
    pub team_type: String,
}
