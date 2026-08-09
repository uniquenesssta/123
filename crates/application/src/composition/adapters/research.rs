use super::super::port_registry::{map_persistence_error, ActiveDatabase};
use crate::ports::{
    research::{ResearchArtifactPort, ResearchEvidenceLedgerPort},
    PortResult,
};
use async_trait::async_trait;
use football_domain::{
    CompetitionProfileVersionDraft, CompetitionProfileVersionRecord, EvidenceClaimDraft,
    EvidenceClaimRecord, EvidenceConflictDraft, EvidenceConflictRecord, PromptVersionDraft,
    PromptVersionRecord, ResearchRunDraft, ResearchRunEventDraft, ResearchRunRecord,
    SchemaVersionDraft, SchemaVersionRecord, SourcePolicyVersionDraft, SourcePolicyVersionRecord,
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
