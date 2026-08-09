use super::super::port_registry::{map_persistence_error, ActiveDatabase};
use crate::ports::{
    research::{
        FactPipelinePort, ResearchArtifactPort, ResearchEvidenceLedgerPort,
        SerializedConflictEventPayload,
    },
    PortError, PortErrorKind, PortResult,
};
use async_trait::async_trait;
use football_domain::{
    CompetitionProfileVersionDraft, CompetitionProfileVersionRecord, ConflictEvaluationDraft,
    ConflictEvaluationRecord, EntityCandidate, EntityResolutionDraft, EntityResolutionRecord,
    EvidenceClaimDraft, EvidenceClaimRecord, EvidenceConflictDraft, EvidenceConflictRecord,
    EvidenceRouteDraft, EvidenceRouteRecord, FactPipelineContext, PromptVersionDraft,
    PromptVersionRecord, ResearchRunDraft, ResearchRunEventDraft, ResearchRunRecord,
    SchemaVersionDraft, SchemaVersionRecord, SourcePolicyVersionDraft, SourcePolicyVersionRecord,
    TimeAuditDraft, TimeAuditRecord,
};
use uuid::Uuid;

#[async_trait]
impl ResearchArtifactPort for ActiveDatabase {
    async fn read_schema(
        &self,
        schema_key: &str,
        version: &str,
    ) -> PortResult<SchemaVersionRecord> {
        self.transition_store()
            .read_schema_version_by_key(schema_key, version)
            .await
            .map_err(map_persistence_error)
    }

    async fn register_schema(&self, draft: &SchemaVersionDraft) -> PortResult<SchemaVersionRecord> {
        self.transition_store()
            .register_schema_version(draft)
            .await
            .map_err(map_persistence_error)
    }

    async fn register_prompt(&self, draft: &PromptVersionDraft) -> PortResult<PromptVersionRecord> {
        self.transition_store()
            .register_prompt_version(draft)
            .await
            .map_err(map_persistence_error)
    }

    async fn register_source_policy(
        &self,
        draft: &SourcePolicyVersionDraft,
    ) -> PortResult<SourcePolicyVersionRecord> {
        self.transition_store()
            .register_source_policy_version(draft)
            .await
            .map_err(map_persistence_error)
    }

    async fn register_competition_profile(
        &self,
        draft: &CompetitionProfileVersionDraft,
    ) -> PortResult<CompetitionProfileVersionRecord> {
        self.transition_store()
            .register_competition_profile_version(draft)
            .await
            .map_err(map_persistence_error)
    }

    async fn create_run(&self, draft: &ResearchRunDraft) -> PortResult<ResearchRunRecord> {
        self.transition_store()
            .create_research_run(draft)
            .await
            .map_err(map_persistence_error)
    }

    async fn read_run(&self, run_id: Uuid) -> PortResult<ResearchRunRecord> {
        self.transition_store()
            .read_research_run(run_id)
            .await
            .map_err(map_persistence_error)
    }

    async fn record_run_event(
        &self,
        draft: &ResearchRunEventDraft,
    ) -> PortResult<ResearchRunRecord> {
        self.transition_store()
            .record_research_run_event(draft)
            .await
            .map_err(map_persistence_error)
    }
}

#[async_trait]
impl ResearchEvidenceLedgerPort for ActiveDatabase {
    async fn append_evidence_claim(
        &self,
        draft: &EvidenceClaimDraft,
    ) -> PortResult<EvidenceClaimRecord> {
        self.transition_store()
            .append_evidence_claim(draft)
            .await
            .map_err(map_persistence_error)
    }

    async fn create_evidence_conflict(
        &self,
        draft: &EvidenceConflictDraft,
    ) -> PortResult<EvidenceConflictRecord> {
        self.transition_store()
            .create_evidence_conflict(draft)
            .await
            .map_err(map_persistence_error)
    }
}

#[async_trait]
impl FactPipelinePort for ActiveDatabase {
    async fn context(&self, research_run_id: Uuid) -> PortResult<FactPipelineContext> {
        self.transition_store()
            .fact_pipeline_context(research_run_id)
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
        self.transition_store()
            .find_entity_candidates(
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
        self.transition_store()
            .append_entity_resolution(draft)
            .await
            .map_err(map_persistence_error)
    }

    async fn append_time_audit(&self, draft: &TimeAuditDraft) -> PortResult<TimeAuditRecord> {
        self.transition_store()
            .append_time_audit(draft)
            .await
            .map_err(map_persistence_error)
    }

    async fn append_conflict_evaluation(
        &self,
        draft: &ConflictEvaluationDraft,
    ) -> PortResult<ConflictEvaluationRecord> {
        self.transition_store()
            .append_conflict_evaluation(draft)
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
        self.transition_store()
            .append_conflict_event(conflict_id, event_type, actor, &payload, idempotency_key)
            .await
            .map_err(map_persistence_error)
    }

    async fn append_evidence_route(
        &self,
        draft: &EvidenceRouteDraft,
    ) -> PortResult<EvidenceRouteRecord> {
        self.transition_store()
            .append_evidence_route(draft)
            .await
            .map_err(map_persistence_error)
    }
}
