//! A second legal task context: traces, research runs and conflict members are unique.
use super::TestDatabase;
use football_domain::{
    ConflictEvaluationDraft, EvidenceClaimDraft, EvidenceConflictDraft, EvidenceConflictRecord,
    EvidenceRouteDraft, ResearchRunDraft, ResearchRunRecord,
};
use uuid::Uuid;

pub(super) struct Drafts<'a> {
    pub research: &'a ResearchRunDraft,
    pub claim: &'a EvidenceClaimDraft,
    pub conflict: &'a EvidenceConflictDraft,
    pub evaluation: &'a ConflictEvaluationDraft,
    pub route: &'a EvidenceRouteDraft,
}

pub(super) struct Peer {
    pub trace: Uuid,
    pub research: ResearchRunRecord,
    pub conflict: EvidenceConflictRecord,
    pub route: EvidenceRouteDraft,
}

pub(super) async fn create(database: &TestDatabase, drafts: Drafts<'_>, token: &str) -> Peer {
    let trace = Uuid::new_v4();
    let mut research = drafts.research.clone();
    research.trace_id = trace;
    research.horizon = football_domain::P4Horizon::T6h;
    research.data_cutoff_at += chrono::Duration::hours(18);
    research.idempotency_key = format!("workbench-peer-run:{token}");
    let research = database.store.create_research_run(&research).await.unwrap();
    let mut ids = Vec::new();
    for suffix in ["a", "b"] {
        let mut claim = drafts.claim.clone();
        claim.match_id = research.match_id;
        claim.research_run_id = research.id;
        claim.idempotency_key = format!("workbench-peer-claim:{token}:{suffix}");
        ids.push(
            database
                .store
                .append_evidence_claim(&claim)
                .await
                .unwrap()
                .id,
        );
    }
    let mut conflict = drafts.conflict.clone();
    conflict.trace_id = trace;
    conflict.evidence_ids = ids.clone();
    conflict.conflict_key = format!("workbench-peer-conflict:{token}");
    let conflict = database
        .store
        .create_evidence_conflict(&conflict)
        .await
        .unwrap();
    let mut evaluation = drafts.evaluation.clone();
    evaluation.conflict_id = conflict.id;
    evaluation.research_run_id = research.id;
    evaluation.trace_id = trace;
    evaluation.idempotency_key = format!("workbench-peer-evaluation:{token}");
    database
        .store
        .append_conflict_evaluation(&evaluation)
        .await
        .unwrap();
    let mut route = drafts.route.clone();
    route.research_run_id = research.id;
    route.trace_id = trace;
    route.route_key = format!("workbench-peer-blocked:{token}");
    route.idempotency_key = format!("workbench-peer-route:{token}");
    route.selected_evidence_ids = ids;
    database.store.append_evidence_route(&route).await.unwrap();
    Peer {
        trace,
        research,
        conflict,
        route,
    }
}
