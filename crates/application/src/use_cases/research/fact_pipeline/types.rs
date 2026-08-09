use super::*;

#[derive(Debug, Clone)]
pub(super) struct SourceReference {
    pub(super) url: String,
    pub(super) title: String,
    pub(super) domain: String,
    pub(super) independence_key: String,
    pub(super) tier: String,
    pub(super) rank: u16,
}

#[derive(Debug, Clone)]
pub(super) struct PreparedFact {
    pub(super) fact: ResearchFact,
    pub(super) normalized_name: String,
    pub(super) resolution: EntityResolutionRecord,
    pub(super) time_audit: TimeAuditRecord,
    pub(super) retrieved_at: DateTime<Utc>,
    pub(super) sources: Vec<SourceReference>,
    pub(super) value: Value,
}

#[derive(Debug, Clone)]
pub(super) struct PersistedFact {
    pub(super) prepared: PreparedFact,
    pub(super) evidence_ids: Vec<Uuid>,
    pub(super) claim_state: EvidenceVerificationState,
}

#[derive(Debug, Clone)]
pub(super) struct RankedValue {
    pub(super) key: String,
    pub(super) value: Value,
    pub(super) evidence_ids: Vec<Uuid>,
    pub(super) max_tier_rank: u16,
    pub(super) independent_domains: usize,
    pub(super) latest_evidence_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone)]
pub(super) struct ResolutionDecision {
    pub(super) status: EntityResolutionStatus,
    pub(super) resolved_entity_id: Option<Uuid>,
    pub(super) resolved_name: Option<String>,
    pub(super) strategy: String,
    pub(super) confidence_score: u16,
    pub(super) reason: String,
}
