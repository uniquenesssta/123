use super::super::port_registry::PersistenceStore;
use super::map_persistence_error;
use crate::ports::{
    research::{
        FactPipelinePort, ResearchArtifactPort, ResearchEvidenceLedgerPort,
        ResearchGatewayAuditPort, ResearchManualConflictPort, SerializedConflictEventPayload,
    },
    PortError, PortErrorKind, PortResult,
};
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

#[async_trait]
impl ResearchArtifactPort for PersistenceStore {
    async fn read_schema(
        &self,
        schema_key: &str,
        version: &str,
    ) -> PortResult<SchemaVersionRecord> {
        self.read_schema_version_by_key(schema_key, version)
            .await
            .map_err(map_persistence_error)
    }

    async fn register_schema(&self, draft: &SchemaVersionDraft) -> PortResult<SchemaVersionRecord> {
        self.register_schema_version(draft)
            .await
            .map_err(map_persistence_error)
    }

    async fn register_prompt(&self, draft: &PromptVersionDraft) -> PortResult<PromptVersionRecord> {
        self.register_prompt_version(draft)
            .await
            .map_err(map_persistence_error)
    }

    async fn register_source_policy(
        &self,
        draft: &SourcePolicyVersionDraft,
    ) -> PortResult<SourcePolicyVersionRecord> {
        self.register_source_policy_version(draft)
            .await
            .map_err(map_persistence_error)
    }

    async fn register_competition_profile(
        &self,
        draft: &CompetitionProfileVersionDraft,
    ) -> PortResult<CompetitionProfileVersionRecord> {
        self.register_competition_profile_version(draft)
            .await
            .map_err(map_persistence_error)
    }

    async fn create_run(&self, draft: &ResearchRunDraft) -> PortResult<ResearchRunRecord> {
        self.create_research_run(draft)
            .await
            .map_err(map_persistence_error)
    }

    async fn read_run(&self, run_id: Uuid) -> PortResult<ResearchRunRecord> {
        self.read_research_run(run_id)
            .await
            .map_err(map_persistence_error)
    }

    async fn record_run_event(
        &self,
        draft: &ResearchRunEventDraft,
    ) -> PortResult<ResearchRunRecord> {
        self.record_research_run_event(draft)
            .await
            .map_err(map_persistence_error)
    }
}

#[async_trait]
impl ResearchEvidenceLedgerPort for PersistenceStore {
    async fn append_evidence_claim(
        &self,
        draft: &EvidenceClaimDraft,
    ) -> PortResult<EvidenceClaimRecord> {
        self.append_evidence_claim(draft)
            .await
            .map_err(map_persistence_error)
    }

    async fn create_evidence_conflict(
        &self,
        draft: &EvidenceConflictDraft,
    ) -> PortResult<EvidenceConflictRecord> {
        self.create_evidence_conflict(draft)
            .await
            .map_err(map_persistence_error)
    }
}

#[async_trait]
impl ResearchManualConflictPort for PersistenceStore {
    async fn append_manual_route_override(
        &self,
        draft: &P4ManualRouteOverrideDraft,
    ) -> PortResult<P4ManualRouteOverrideRecord> {
        self.append_p4_manual_route_override(draft)
            .await
            .map_err(map_persistence_error)
    }

    async fn route_readiness(&self, task_id: Uuid) -> PortResult<P4FreezeReadiness> {
        self.p4_route_readiness(task_id)
            .await
            .map_err(map_persistence_error)
    }
}

#[async_trait]
impl FactPipelinePort for PersistenceStore {
    async fn context(&self, research_run_id: Uuid) -> PortResult<FactPipelineContext> {
        self.fact_pipeline_context(research_run_id)
            .await
            .map_err(map_persistence_error)
    }

    async fn find_entity_candidates(
        &self,
        context: &FactPipelineContext,
        entity_type: &str,
        normalized_name: &str,
        compact_name: &str,
        external_id: Option<&str>,
    ) -> PortResult<Vec<EntityCandidate>> {
        self.find_entity_candidates(
            context,
            entity_type,
            normalized_name,
            compact_name,
            external_id,
        )
        .await
        .map_err(map_persistence_error)
    }

    async fn append_entity_resolution(
        &self,
        draft: &EntityResolutionDraft,
    ) -> PortResult<EntityResolutionRecord> {
        self.append_entity_resolution(draft)
            .await
            .map_err(map_persistence_error)
    }

    async fn append_time_audit(&self, draft: &TimeAuditDraft) -> PortResult<TimeAuditRecord> {
        self.append_time_audit(draft)
            .await
            .map_err(map_persistence_error)
    }

    async fn append_conflict_evaluation(
        &self,
        draft: &ConflictEvaluationDraft,
    ) -> PortResult<ConflictEvaluationRecord> {
        self.append_conflict_evaluation(draft)
            .await
            .map_err(map_persistence_error)
    }

    async fn append_conflict_event(
        &self,
        conflict_id: Uuid,
        event_type: &str,
        actor: &str,
        payload: &SerializedConflictEventPayload,
        idempotency_key: &str,
    ) -> PortResult<()> {
        let payload = serde_json::from_str(&payload.0).map_err(|error| {
            PortError::new(
                PortErrorKind::Serialization,
                format!("冲突事件载荷反序列化失败：{error}"),
            )
        })?;
        self.append_conflict_event(conflict_id, event_type, actor, &payload, idempotency_key)
            .await
            .map_err(map_persistence_error)
    }

    async fn append_evidence_route(
        &self,
        draft: &EvidenceRouteDraft,
    ) -> PortResult<EvidenceRouteRecord> {
        self.append_evidence_route(draft)
            .await
            .map_err(map_persistence_error)
    }
}

#[async_trait]
impl ResearchGatewayAuditPort for PersistenceStore {
    async fn append_attempt(&self, draft: &OpenAiAttemptDraft) -> PortResult<OpenAiAttemptRecord> {
        self.append_openai_attempt(draft)
            .await
            .map_err(map_persistence_error)
    }

    async fn attempt_number_offset(&self, research_run_id: Uuid) -> PortResult<u32> {
        self.openai_attempt_number_offset(research_run_id)
            .await
            .map_err(map_persistence_error)
    }

    async fn usage_totals(&self) -> PortResult<OpenAiUsageTotals> {
        self.openai_usage_totals()
            .await
            .map_err(map_persistence_error)
    }

    async fn append_web_references(
        &self,
        _run_id: Uuid,
        sources: &[WebSourceDraft],
        citations: &[WebCitationDraft],
    ) -> PortResult<()> {
        self.append_web_references(citations, sources)
            .await
            .map_err(map_persistence_error)
    }
}
