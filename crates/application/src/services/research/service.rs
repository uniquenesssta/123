use crate::ports::research::{
    ResearchArtifactPort, ResearchEvidenceLedgerPort, ResearchGatewayAuditPort,
};
use crate::use_cases::research::{
    artifact_catalog,
    fact_pipeline::{self, FactPipelineAccess, ProcessResearchEvidenceCommand},
    ledger,
    openai_gateway::{self, OpenAiResearchCommand},
    p4_manual_conflict::{self, P4ManualConflictAccess},
    p4_worker::{self, P4ResearchWorkerAccess},
};
use crate::ApplicationResult;
use football_domain::{
    CompetitionProfileVersionDraft, CompetitionProfileVersionRecord, EvidenceClaimDraft,
    EvidenceClaimRecord, EvidenceConflictDraft, EvidenceConflictRecord, FactPipelineSummary,
    P4TaskWorkspace, PromptVersionDraft, PromptVersionRecord, ResearchRunDraft,
    ResearchRunEventDraft, ResearchRunRecord, ResolveP4ConflictCommand, SchemaVersionDraft,
    SchemaVersionRecord,
};
use football_research_gateway::{CancellationToken, GatewayExecution};
use serde_json::Value;
use uuid::Uuid;

pub(crate) struct ResearchService;

impl ResearchService {
    pub(crate) fn new() -> Self {
        Self
    }

    pub(crate) async fn register_persistence_artifacts<P: ResearchArtifactPort + ?Sized>(
        &self,
        port: &P,
    ) -> ApplicationResult<()> {
        artifact_catalog::register_built_ins(port).await
    }

    pub(crate) async fn register_p4_schema_version<P: ResearchArtifactPort + ?Sized>(
        &self,
        port: &P,
        draft: SchemaVersionDraft,
    ) -> ApplicationResult<SchemaVersionRecord> {
        artifact_catalog::register_schema(port, draft).await
    }

    pub(crate) async fn register_p4_prompt_version<P: ResearchArtifactPort + ?Sized>(
        &self,
        port: &P,
        draft: PromptVersionDraft,
    ) -> ApplicationResult<PromptVersionRecord> {
        artifact_catalog::register_prompt(port, draft).await
    }

    pub(crate) async fn register_p4_competition_profile_version<
        P: ResearchArtifactPort + ?Sized,
    >(
        &self,
        port: &P,
        draft: CompetitionProfileVersionDraft,
    ) -> ApplicationResult<CompetitionProfileVersionRecord> {
        artifact_catalog::register_competition_profile(port, draft).await
    }

    pub(crate) async fn create_p4_research_run<P: ResearchArtifactPort + ?Sized>(
        &self,
        port: &P,
        draft: ResearchRunDraft,
    ) -> ApplicationResult<ResearchRunRecord> {
        ledger::create_run(port, draft).await
    }

    pub(crate) async fn record_p4_research_run_event<P: ResearchArtifactPort + ?Sized>(
        &self,
        port: &P,
        draft: ResearchRunEventDraft,
    ) -> ApplicationResult<ResearchRunRecord> {
        ledger::record_run_event(port, draft).await
    }

    pub(crate) async fn append_p4_evidence_claim<P: ResearchEvidenceLedgerPort + ?Sized>(
        &self,
        port: &P,
        draft: EvidenceClaimDraft,
    ) -> ApplicationResult<EvidenceClaimRecord> {
        ledger::append_evidence_claim(port, draft).await
    }

    pub(crate) async fn create_p4_evidence_conflict<P: ResearchEvidenceLedgerPort + ?Sized>(
        &self,
        port: &P,
        draft: EvidenceConflictDraft,
    ) -> ApplicationResult<EvidenceConflictRecord> {
        ledger::create_evidence_conflict(port, draft).await
    }

    pub(crate) async fn process_p4_research_evidence(
        &self,
        port: &dyn FactPipelineAccess,
        command: ProcessResearchEvidenceCommand,
    ) -> ApplicationResult<FactPipelineSummary> {
        fact_pipeline::process_p4_research_evidence(port, command).await
    }

    pub(crate) async fn register_openai_research_artifacts(
        &self,
        port: &dyn ResearchArtifactPort,
    ) -> ApplicationResult<()> {
        openai_gateway::register_openai_research_artifacts(port).await
    }

    pub(crate) async fn execute_p4_openai_research(
        &self,
        artifacts: &dyn ResearchArtifactPort,
        audit: &dyn ResearchGatewayAuditPort,
        pipeline: &dyn FactPipelineAccess,
        command: OpenAiResearchCommand,
        cancellation: CancellationToken,
    ) -> ApplicationResult<GatewayExecution> {
        openai_gateway::execute_p4_openai_research(
            artifacts,
            audit,
            pipeline,
            command,
            cancellation,
        )
        .await
    }

    pub(crate) async fn execute_p4_research_task(
        &self,
        access: P4ResearchWorkerAccess<'_>,
        task_id: Uuid,
        job_id: Uuid,
    ) -> ApplicationResult<Value> {
        p4_worker::execute(access, task_id, job_id).await
    }

    pub(crate) async fn resolve_p4_conflict(
        &self,
        access: P4ManualConflictAccess<'_>,
        command: ResolveP4ConflictCommand,
    ) -> ApplicationResult<P4TaskWorkspace> {
        p4_manual_conflict::resolve(access, command).await
    }
}
