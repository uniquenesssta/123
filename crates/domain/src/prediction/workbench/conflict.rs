use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum P4ManualConflictDecisionKind {
    SelectEvidence,
    AcceptUnknown,
}

impl P4ManualConflictDecisionKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::SelectEvidence => "select_evidence",
            Self::AcceptUnknown => "accept_unknown",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolveP4ConflictCommand {
    pub task_id: Uuid,
    pub conflict_id: Uuid,
    pub decision_kind: P4ManualConflictDecisionKind,
    #[serde(default)]
    pub selected_evidence_ids: Vec<Uuid>,
    #[serde(default)]
    pub note: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct P4ManualRouteOverrideDraft {
    pub task_id: Uuid,
    pub research_run_id: Uuid,
    pub conflict_id: Uuid,
    pub route_key: String,
    pub field_key: String,
    pub target_module: String,
    pub target_slot: String,
    #[serde(default)]
    pub entity_type: Option<String>,
    #[serde(default)]
    pub entity_id: Option<Uuid>,
    pub decision_kind: P4ManualConflictDecisionKind,
    #[serde(default)]
    pub selected_evidence_ids: Vec<Uuid>,
    #[serde(default)]
    pub selected_value: Value,
    pub verification_state: String,
    pub route_status: String,
    pub reason: String,
    pub actor: String,
    #[serde(default)]
    pub note: Option<String>,
    pub idempotency_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct P4ManualRouteOverrideRecord {
    pub id: Uuid,
    pub task_id: Uuid,
    pub conflict_id: Uuid,
    pub route_key: String,
    pub decision_kind: P4ManualConflictDecisionKind,
    pub selected_evidence_ids: Vec<Uuid>,
    pub route_status: String,
    pub verification_state: String,
    pub created_at: DateTime<Utc>,
}
