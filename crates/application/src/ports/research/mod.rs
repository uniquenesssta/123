use crate::ports::PortResult;
use async_trait::async_trait;
use football_domain::{
    ConflictEvaluationDraft, ConflictEvaluationRecord, EntityMatchRequest, EntityResolutionDraft,
    EntityResolutionRecord, EvidenceRouteDraft, EvidenceRouteRecord, FactPipelineContext,
    OpenAiAttemptDraft, OpenAiAttemptRecord, OpenAiUsageTotals, PromptVersionDraft,
    PromptVersionRecord, ResearchRunDraft, ResearchRunEventDraft, ResearchRunRecord,
    SchemaVersionDraft, SchemaVersionRecord, SourcePolicyVersionDraft, SourcePolicyVersionRecord,
    TimeAuditDraft, TimeAuditRecord, WebCitationDraft, WebSourceDraft,
};
use uuid::Uuid;

#[async_trait]
pub trait ResearchArtifactPort: Send + Sync {
    async fn register_schema(&self, draft: &SchemaVersionDraft) -> PortResult<SchemaVersionRecord>;
    async fn register_prompt(&self, draft: &PromptVersionDraft) -> PortResult<PromptVersionRecord>;
    async fn register_source_policy(
        &self,
        draft: &SourcePolicyVersionDraft,
    ) -> PortResult<SourcePolicyVersionRecord>;
    async fn create_run(&self, draft: &ResearchRunDraft) -> PortResult<ResearchRunRecord>;
    async fn read_run(&self, run_id: Uuid) -> PortResult<ResearchRunRecord>;
    async fn record_run_event(&self, draft: &ResearchRunEventDraft) -> PortResult<()>;
}

#[async_trait]
pub trait FactPipelinePort: Send + Sync {
    async fn context(&self, match_id: Uuid) -> PortResult<FactPipelineContext>;
    async fn resolve_entity(
        &self,
        request: &EntityMatchRequest,
    ) -> PortResult<EntityResolutionRecord>;
    async fn append_entity_resolution(
        &self,
        draft: &EntityResolutionDraft,
    ) -> PortResult<EntityResolutionRecord>;
    async fn append_time_audit(&self, draft: &TimeAuditDraft) -> PortResult<TimeAuditRecord>;
    async fn append_conflict_evaluation(
        &self,
        draft: &ConflictEvaluationDraft,
    ) -> PortResult<ConflictEvaluationRecord>;
    async fn append_evidence_route(
        &self,
        draft: &EvidenceRouteDraft,
    ) -> PortResult<EvidenceRouteRecord>;
}

#[async_trait]
pub trait ResearchGatewayAuditPort: Send + Sync {
    async fn append_attempt(&self, draft: &OpenAiAttemptDraft) -> PortResult<OpenAiAttemptRecord>;
    async fn usage_totals(&self) -> PortResult<OpenAiUsageTotals>;
    async fn append_web_references(
        &self,
        run_id: Uuid,
        sources: &[WebSourceDraft],
        citations: &[WebCitationDraft],
    ) -> PortResult<()>;
}
