use super::default_coach_status;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoachDraft {
    pub canonical_name: String,
    #[serde(default)]
    pub nationality_code: Option<String>,
    #[serde(default = "default_coach_status")]
    pub status: String,
    #[serde(default)]
    pub metadata: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoachRecord {
    pub id: Uuid,
    pub canonical_name: String,
    pub normalized_name: String,
    pub nationality_code: Option<String>,
    pub status: String,
    pub metadata: Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
