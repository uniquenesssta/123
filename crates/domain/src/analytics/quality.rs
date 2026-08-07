use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

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
