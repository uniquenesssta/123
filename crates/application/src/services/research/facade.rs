use crate::composition::ActiveDatabase;
use crate::use_cases::research::p4_manual_conflict::P4ManualConflictAccess;
use crate::{
    ApplicationError, ApplicationResult, ApplicationService, OpenAiResearchCommand,
    ProcessResearchEvidenceCommand,
};
use football_domain::{
    CompetitionProfileVersionDraft, CompetitionProfileVersionRecord, EvidenceClaimDraft,
    EvidenceClaimRecord, EvidenceConflictDraft, EvidenceConflictRecord, FactPipelineSummary,
    P4TaskWorkspace, PromptVersionDraft, PromptVersionRecord, ResearchRunDraft,
    ResearchRunEventDraft, ResearchRunRecord, ResolveP4ConflictCommand, SchemaVersionDraft,
    SchemaVersionRecord,
};
use football_research_gateway::{CancellationToken, GatewayExecution};

impl ApplicationService {
    async fn research_session(&self) -> ApplicationResult<ActiveDatabase> {
        self.database
            .active_session()
            .await
            .ok_or(ApplicationError::DatabaseNotConnected)
    }

    pub async fn register_p4_schema_version(
        &self,
        draft: SchemaVersionDraft,
    ) -> ApplicationResult<SchemaVersionRecord> {
        let session = self.research_session().await?;
        self.research
            .register_p4_schema_version(&session, draft)
            .await
    }

    pub async fn register_p4_prompt_version(
        &self,
        draft: PromptVersionDraft,
    ) -> ApplicationResult<PromptVersionRecord> {
        let session = self.research_session().await?;
        self.research
            .register_p4_prompt_version(&session, draft)
            .await
    }

    pub async fn register_p4_competition_profile_version(
        &self,
        draft: CompetitionProfileVersionDraft,
    ) -> ApplicationResult<CompetitionProfileVersionRecord> {
        let session = self.research_session().await?;
        self.research
            .register_p4_competition_profile_version(&session, draft)
            .await
    }

    pub async fn create_p4_research_run(
        &self,
        draft: ResearchRunDraft,
    ) -> ApplicationResult<ResearchRunRecord> {
        let session = self.research_session().await?;
        self.research.create_p4_research_run(&session, draft).await
    }

    pub async fn record_p4_research_run_event(
        &self,
        draft: ResearchRunEventDraft,
    ) -> ApplicationResult<ResearchRunRecord> {
        let session = self.research_session().await?;
        self.research
            .record_p4_research_run_event(&session, draft)
            .await
    }

    pub async fn append_p4_evidence_claim(
        &self,
        draft: EvidenceClaimDraft,
    ) -> ApplicationResult<EvidenceClaimRecord> {
        let session = self.research_session().await?;
        self.research
            .append_p4_evidence_claim(&session, draft)
            .await
    }

    pub async fn create_p4_evidence_conflict(
        &self,
        draft: EvidenceConflictDraft,
    ) -> ApplicationResult<EvidenceConflictRecord> {
        let session = self.research_session().await?;
        self.research
            .create_p4_evidence_conflict(&session, draft)
            .await
    }

    pub async fn process_p4_research_evidence(
        &self,
        command: ProcessResearchEvidenceCommand,
    ) -> ApplicationResult<FactPipelineSummary> {
        let session = self.research_session().await?;
        self.research
            .process_p4_research_evidence(&session, command)
            .await
    }

    pub async fn execute_p4_openai_research(
        &self,
        command: OpenAiResearchCommand,
        cancellation: CancellationToken,
    ) -> ApplicationResult<GatewayExecution> {
        let session = self.research_session().await?;
        self.research
            .execute_p4_openai_research(&session, &session, &session, command, cancellation)
            .await
    }

    pub async fn resolve_p4_conflict(
        &self,
        command: ResolveP4ConflictCommand,
    ) -> ApplicationResult<P4TaskWorkspace> {
        let session = self.research_session().await?;
        let access = P4ManualConflictAccess {
            workflow: &session,
            jobs: &session,
            artifacts: &session,
            manual: &session,
        };
        self.research.resolve_p4_conflict(access, command).await
    }
}
