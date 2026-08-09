use crate::ports::research::{ResearchArtifactPort, ResearchEvidenceLedgerPort};
use crate::ApplicationResult;
use football_domain::{
    EvidenceClaimDraft, EvidenceClaimRecord, EvidenceConflictDraft, EvidenceConflictRecord,
    ResearchRunDraft, ResearchRunEventDraft, ResearchRunRecord,
};

pub(crate) async fn create_run<P: ResearchArtifactPort + ?Sized>(
    port: &P,
    draft: ResearchRunDraft,
) -> ApplicationResult<ResearchRunRecord> {
    Ok(port.create_run(&draft).await?)
}

pub(crate) async fn record_run_event<P: ResearchArtifactPort + ?Sized>(
    port: &P,
    draft: ResearchRunEventDraft,
) -> ApplicationResult<ResearchRunRecord> {
    Ok(port.record_run_event(&draft).await?)
}

pub(crate) async fn append_evidence_claim<P: ResearchEvidenceLedgerPort + ?Sized>(
    port: &P,
    draft: EvidenceClaimDraft,
) -> ApplicationResult<EvidenceClaimRecord> {
    Ok(port.append_evidence_claim(&draft).await?)
}

pub(crate) async fn create_evidence_conflict<P: ResearchEvidenceLedgerPort + ?Sized>(
    port: &P,
    draft: EvidenceConflictDraft,
) -> ApplicationResult<EvidenceConflictRecord> {
    Ok(port.create_evidence_conflict(&draft).await?)
}
