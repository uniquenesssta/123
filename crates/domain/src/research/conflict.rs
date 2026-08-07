use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum ConflictEvaluationStatus {
    AutoResolved,
    ManualRequired,
    AcceptedUnknown,
}

impl ConflictEvaluationStatus {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::AutoResolved => "auto_resolved",
            Self::ManualRequired => "manual_required",
            Self::AcceptedUnknown => "accepted_unknown",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictEvaluationDraft {
    pub conflict_id: Uuid,
    pub research_run_id: Uuid,
    pub match_id: Uuid,
    pub trace_id: Uuid,
    pub source_policy_key: String,
    pub source_policy_version: String,
    pub status: ConflictEvaluationStatus,
    #[serde(default)]
    pub winning_evidence_ids: Vec<Uuid>,
    #[serde(default)]
    pub winning_value: Value,
    pub ranking: Value,
    pub reason: String,
    pub idempotency_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictEvaluationRecord {
    pub id: Uuid,
    pub status: ConflictEvaluationStatus,
    pub evaluation_fingerprint: String,
    pub created_at: DateTime<Utc>,
}
