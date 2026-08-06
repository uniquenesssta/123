use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoundDraft {
    pub stage_id: Uuid,
    pub code: String,
    pub name: String,
    #[serde(default)]
    pub sequence_no: i32,
    #[serde(default)]
    pub starts_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub ends_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoundRecord {
    pub id: Uuid,
    pub stage_id: Uuid,
    pub stage_name: String,
    pub code: String,
    pub name: String,
    pub sequence_no: i32,
    pub starts_at: Option<DateTime<Utc>>,
    pub ends_at: Option<DateTime<Utc>>,
}
