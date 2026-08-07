use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceRouteStatus {
    Routed,
    Missing,
    BlockedEntity,
    BlockedTime,
    BlockedConflict,
    BlockedUnregisteredField,
    IgnoredNonModelFact,
}

impl EvidenceRouteStatus {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Routed => "routed",
            Self::Missing => "missing",
            Self::BlockedEntity => "blocked_entity",
            Self::BlockedTime => "blocked_time",
            Self::BlockedConflict => "blocked_conflict",
            Self::BlockedUnregisteredField => "blocked_unregistered_field",
            Self::IgnoredNonModelFact => "ignored_non_model_fact",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EvidenceRouteRule {
    pub field_key: String,
    pub target_module: String,
    pub target_slot: String,
    pub entity_type: String,
    #[serde(default)]
    pub side: Option<String>,
    #[serde(default = "default_true")]
    pub requires_resolved_entity: bool,
    #[serde(default)]
    pub allow_multiple_entities: bool,
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EvidenceRouteRegistry {
    pub schema_version: String,
    pub routes: Vec<EvidenceRouteRule>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceRouteDraft {
    pub research_run_id: Uuid,
    pub match_id: Uuid,
    pub trace_id: Uuid,
    pub route_key: String,
    pub field_key: String,
    pub target_module: String,
    pub target_slot: String,
    pub route_registry_version: String,
    #[serde(default)]
    pub entity_type: Option<String>,
    #[serde(default)]
    pub entity_id: Option<Uuid>,
    pub status: EvidenceRouteStatus,
    pub verification_state: String,
    #[serde(default)]
    pub selected_evidence_ids: Vec<Uuid>,
    #[serde(default)]
    pub selected_value: Value,
    pub reason: String,
    pub idempotency_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceRouteRecord {
    pub id: Uuid,
    pub status: EvidenceRouteStatus,
    pub route_fingerprint: String,
    pub created_at: DateTime<Utc>,
}
