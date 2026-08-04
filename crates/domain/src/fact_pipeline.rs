use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

pub const P4_FACT_PIPELINE_CONTRACT_VERSION: &str = "football.p4-fact-pipeline.v1";
pub const P4_SOURCE_POLICY_VERSION: &str = "football.p4-source-policy.v1";
pub const P4_EVIDENCE_ROUTE_VERSION: &str = "football.p4-evidence-routes.v1";

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum EntityResolutionStatus {
    Resolved,
    Ambiguous,
    Unmatched,
    Unsupported,
}

impl EntityResolutionStatus {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Resolved => "resolved",
            Self::Ambiguous => "ambiguous",
            Self::Unmatched => "unmatched",
            Self::Unsupported => "unsupported",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EntityCandidate {
    pub entity_id: Uuid,
    pub canonical_name: String,
    pub matched_name: String,
    pub strategy: String,
    pub score: u16,
    #[serde(default)]
    pub relation: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityResolutionDraft {
    pub research_run_id: Uuid,
    pub match_id: Uuid,
    pub trace_id: Uuid,
    pub fact_key: String,
    pub entity_type: String,
    pub raw_name: String,
    pub normalized_name: String,
    #[serde(default)]
    pub external_id: Option<String>,
    pub status: EntityResolutionStatus,
    #[serde(default)]
    pub resolved_entity_id: Option<Uuid>,
    #[serde(default)]
    pub resolved_name: Option<String>,
    pub strategy: String,
    pub confidence_score: u16,
    #[serde(default)]
    pub candidates: Vec<EntityCandidate>,
    pub reason: String,
    pub idempotency_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityResolutionRecord {
    pub id: Uuid,
    pub status: EntityResolutionStatus,
    pub resolved_entity_id: Option<Uuid>,
    pub resolution_fingerprint: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum TimeAuditStatus {
    Accepted,
    AcceptedNonFact,
    RejectedFuture,
    RejectedRetrievedAfterCutoff,
    RejectedMissingEvidenceTime,
    RejectedMissingTimezone,
    RejectedInvalidOrder,
}

impl TimeAuditStatus {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Accepted => "accepted",
            Self::AcceptedNonFact => "accepted_non_fact",
            Self::RejectedFuture => "rejected_future",
            Self::RejectedRetrievedAfterCutoff => "rejected_retrieved_after_cutoff",
            Self::RejectedMissingEvidenceTime => "rejected_missing_evidence_time",
            Self::RejectedMissingTimezone => "rejected_missing_timezone",
            Self::RejectedInvalidOrder => "rejected_invalid_order",
        }
    }

    pub const fn accepted(self) -> bool {
        matches!(self, Self::Accepted | Self::AcceptedNonFact)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeAuditDraft {
    pub research_run_id: Uuid,
    pub match_id: Uuid,
    pub trace_id: Uuid,
    pub fact_key: String,
    pub field_key: String,
    pub data_cutoff_at: DateTime<Utc>,
    #[serde(default)]
    pub published_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub observed_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub effective_at: Option<DateTime<Utc>>,
    pub retrieved_at: DateTime<Utc>,
    #[serde(default)]
    pub timezone: Option<String>,
    pub status: TimeAuditStatus,
    pub reason: String,
    pub idempotency_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeAuditRecord {
    pub id: Uuid,
    pub status: TimeAuditStatus,
    pub time_fingerprint: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SourceTierRule {
    pub domain: String,
    pub tier: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SourceTierDefinition {
    pub key: String,
    pub rank: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SourcePolicyDefinition {
    pub schema_version: String,
    pub default_tier: String,
    pub tiers: Vec<SourceTierDefinition>,
    pub domain_rules: Vec<SourceTierRule>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourcePolicyVersionDraft {
    pub policy_key: String,
    pub version: String,
    #[serde(default)]
    pub competition_profile_id: Option<Uuid>,
    pub definition: SourcePolicyDefinition,
    #[serde(default)]
    pub metadata: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourcePolicyVersionRecord {
    pub id: Uuid,
    pub policy_key: String,
    pub version: String,
    pub content_sha256: String,
    pub created_at: DateTime<Utc>,
}

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FactPipelineContext {
    pub research_run_id: Uuid,
    pub match_id: Uuid,
    pub match_key: String,
    pub horizon: String,
    pub data_cutoff_at: DateTime<Utc>,
    pub trace_id: Uuid,
    pub prompt_version_id: Option<Uuid>,
    pub prompt_version: Option<String>,
    pub schema_version_id: Uuid,
    pub schema_version: String,
    pub home_team_id: Option<Uuid>,
    pub home_team_name: Option<String>,
    pub away_team_id: Option<Uuid>,
    pub away_team_name: Option<String>,
    pub competition_id: Option<Uuid>,
    pub competition_name: Option<String>,
    pub competition_code: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FactPipelineSummary {
    pub fact_count: u32,
    pub missing_field_count: u32,
    pub evidence_claim_count: u32,
    pub resolved_entity_count: u32,
    pub ambiguous_entity_count: u32,
    pub unmatched_entity_count: u32,
    pub time_rejected_count: u32,
    pub conflict_count: u32,
    pub auto_resolved_conflict_count: u32,
    pub manual_conflict_count: u32,
    pub routed_count: u32,
    pub blocked_count: u32,
}

impl FactPipelineSummary {
    pub const fn has_blockers(&self) -> bool {
        self.ambiguous_entity_count > 0
            || self.unmatched_entity_count > 0
            || self.time_rejected_count > 0
            || self.manual_conflict_count > 0
            || self.blocked_count > 0
    }
}
