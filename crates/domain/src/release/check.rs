use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

use super::contract::ReleaseAcceptanceStatus;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReleaseAcceptanceCheck {
    pub id: Uuid,
    pub run_id: Uuid,
    pub sequence_no: i32,
    pub category: String,
    pub check_code: String,
    pub title: String,
    pub status: ReleaseAcceptanceStatus,
    pub summary: String,
    pub remediation: Option<String>,
    pub evidence: Value,
    pub duration_ms: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ReleaseAcceptanceCategorySummary {
    pub category: String,
    pub passed: u32,
    pub warnings: u32,
    pub blocked: u32,
}
