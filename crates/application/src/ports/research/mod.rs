use crate::ports::PortResult;
use async_trait::async_trait;
use football_domain::{
    CompetitionProfileVersionDraft, CompetitionProfileVersionRecord, ConflictEvaluationDraft,
    ConflictEvaluationRecord, EntityCandidate, EntityResolutionDraft, EntityResolutionRecord,
    EvidenceClaimDraft, EvidenceClaimRecord, EvidenceConflictDraft, EvidenceConflictRecord,
    EvidenceRouteDraft, EvidenceRouteRecord, FactPipelineContext, OpenAiAttemptDraft,
    OpenAiAttemptRecord, OpenAiUsageTotals, P4FreezeReadiness, P4ManualRouteOverrideDraft,
    P4ManualRouteOverrideRecord, PromptVersionDraft, PromptVersionRecord, ResearchRunDraft,
    ResearchRunEventDraft, ResearchRunRecord, SchemaVersionDraft, SchemaVersionRecord,
    SourcePolicyVersionDraft, SourcePolicyVersionRecord, TimeAuditDraft, TimeAuditRecord,
    WebCitationDraft, WebSourceDraft,
};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SerializedConflictEventPayload(pub String);

#[async_trait]
pub trait ResearchArtifactPort: Send + Sync {
    async fn read_schema(&self, schema_key: &str, version: &str)
        -> PortResult<SchemaVersionRecord>;
    async fn register_schema(&self, draft: &SchemaVersionDraft) -> PortResult<SchemaVersionRecord>;
    async fn register_prompt(&self, draft: &PromptVersionDraft) -> PortResult<PromptVersionRecord>;
    async fn register_source_policy(
        &self,
        draft: &SourcePolicyVersionDraft,
    ) -> PortResult<SourcePolicyVersionRecord>;
    async fn register_competition_profile(
        &self,
        draft: &CompetitionProfileVersionDraft,
    ) -> PortResult<CompetitionProfileVersionRecord>;
    async fn create_run(&self, draft: &ResearchRunDraft) -> PortResult<ResearchRunRecord>;
    async fn read_run(&self, run_id: Uuid) -> PortResult<ResearchRunRecord>;
    async fn record_run_event(
        &self,
        draft: &ResearchRunEventDraft,
    ) -> PortResult<ResearchRunRecord>;
}

#[async_trait]
pub trait ResearchEvidenceLedgerPort: Send + Sync {
    async fn append_evidence_claim(
        &self,
        draft: &EvidenceClaimDraft,
    ) -> PortResult<EvidenceClaimRecord>;
    async fn create_evidence_conflict(
        &self,
        draft: &EvidenceConflictDraft,
    ) -> PortResult<EvidenceConflictRecord>;
}

#[async_trait]
pub trait ResearchManualConflictPort: Send + Sync {
    async fn append_manual_route_override(
        &self,
        draft: &P4ManualRouteOverrideDraft,
    ) -> PortResult<P4ManualRouteOverrideRecord>;
    async fn route_readiness(&self, task_id: Uuid) -> PortResult<P4FreezeReadiness>;
}

#[async_trait]
pub trait FactPipelinePort: Send + Sync {
    async fn context(&self, research_run_id: Uuid) -> PortResult<FactPipelineContext>;
    async fn find_entity_candidates(
        &self,
        context: &FactPipelineContext,
        entity_type: &str,
        normalized_name: &str,
        compact_name: &str,
        external_id: Option<&str>,
    ) -> PortResult<Vec<EntityCandidate>>;
    async fn append_entity_resolution(
        &self,
        draft: &EntityResolutionDraft,
    ) -> PortResult<EntityResolutionRecord>;
    async fn append_time_audit(&self, draft: &TimeAuditDraft) -> PortResult<TimeAuditRecord>;
    async fn append_conflict_evaluation(
        &self,
        draft: &ConflictEvaluationDraft,
    ) -> PortResult<ConflictEvaluationRecord>;
    async fn append_conflict_event(
        &self,
        conflict_id: Uuid,
        event_type: &str,
        actor: &str,
        payload: &SerializedConflictEventPayload,
        idempotency_key: &str,
    ) -> PortResult<()>;
    async fn append_evidence_route(
        &self,
        draft: &EvidenceRouteDraft,
    ) -> PortResult<EvidenceRouteRecord>;
}

#[async_trait]
pub trait ResearchGatewayAuditPort: Send + Sync {
    async fn append_attempt(&self, draft: &OpenAiAttemptDraft) -> PortResult<OpenAiAttemptRecord>;
    async fn attempt_number_offset(&self, research_run_id: Uuid) -> PortResult<u32>;
    async fn usage_totals(&self) -> PortResult<OpenAiUsageTotals>;
    async fn append_web_references(
        &self,
        run_id: Uuid,
        sources: &[WebSourceDraft],
        citations: &[WebCitationDraft],
    ) -> PortResult<()>;
}
