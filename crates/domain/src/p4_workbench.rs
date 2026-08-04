use crate::{
    P4FreezeReadiness, P4FreezeTaskEventRecord, P4FreezeTaskRecord, P4RoutedFact,
    PrematchSnapshotBundle,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

pub const P4_WORKBENCH_CONTRACT_VERSION: &str = "football.p4-workbench-contract.v1";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct P4MatchWorkspace {
    pub match_id: Uuid,
    pub match_key: String,
    pub home_team_name: String,
    pub away_team_name: String,
    pub kickoff_at: DateTime<Utc>,
    #[serde(default)]
    pub competition_name: Option<String>,
    pub tasks: Vec<P4FreezeTaskRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct P4ResearchRunWorkspace {
    pub id: Uuid,
    pub status: String,
    pub attempt_count: i32,
    #[serde(default)]
    pub response_id: Option<String>,
    #[serde(default)]
    pub model_id: Option<String>,
    #[serde(default)]
    pub error_category: Option<String>,
    #[serde(default)]
    pub error_message: Option<String>,
    pub created_at: DateTime<Utc>,
    #[serde(default)]
    pub started_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub finished_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct P4EvidenceWorkspaceRecord {
    pub id: Uuid,
    pub field_key: String,
    pub entity_type: String,
    #[serde(default)]
    pub entity_id: Option<Uuid>,
    pub value: Value,
    pub verification_state: String,
    pub source_tier: String,
    #[serde(default)]
    pub source_url: Option<String>,
    #[serde(default)]
    pub source_title: Option<String>,
    #[serde(default)]
    pub source_domain: Option<String>,
    #[serde(default)]
    pub published_at: Option<DateTime<Utc>>,
    pub observed_at: DateTime<Utc>,
    #[serde(default)]
    pub effective_at: Option<DateTime<Utc>>,
    pub retrieved_at: DateTime<Utc>,
    pub timezone: String,
    #[serde(default)]
    pub conflict_group_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct P4ConflictWorkspaceRecord {
    pub id: Uuid,
    pub field_key: String,
    pub entity_type: String,
    #[serde(default)]
    pub entity_id: Option<Uuid>,
    pub conflict_key: String,
    pub status: String,
    #[serde(default)]
    pub evaluation_status: Option<String>,
    pub evidence_ids: Vec<Uuid>,
    #[serde(default)]
    pub selected_evidence_ids: Vec<Uuid>,
    #[serde(default)]
    pub manual_decision_kind: Option<String>,
    #[serde(default)]
    pub manual_decision_note: Option<String>,
    #[serde(default)]
    pub manual_decision_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct P4TaskWorkspace {
    pub task: P4FreezeTaskRecord,
    pub readiness: P4FreezeReadiness,
    pub events: Vec<P4FreezeTaskEventRecord>,
    #[serde(default)]
    pub research_run: Option<P4ResearchRunWorkspace>,
    pub routes: Vec<P4RoutedFact>,
    pub evidence: Vec<P4EvidenceWorkspaceRecord>,
    pub conflicts: Vec<P4ConflictWorkspaceRecord>,
    #[serde(default)]
    pub snapshot: Option<PrematchSnapshotBundle>,
}

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
