use super::{decide_evidence_scoring_item, list_evidence_scoring_items};
use crate::{
    ports::{postmatch::PostmatchSettlementPort, PortError, PortErrorKind, PortResult},
    ApplicationError,
};
use async_trait::async_trait;
use football_domain::{
    EvidenceScoringDecisionDraft, EvidenceScoringItemRecord, EvidenceVerdict,
    PostmatchSettlementDraft, PostmatchSettlementReadiness, PostmatchSettlementRecord,
};
use serde_json::json;
use std::sync::Mutex;
use uuid::Uuid;

fn id(value: u128) -> Uuid {
    Uuid::from_u128(value)
}

fn item_record() -> EvidenceScoringItemRecord {
    serde_json::from_value(json!({
        "id": id(1), "settlement_id": id(2), "evidence_id": id(3), "provider_id": null,
        "provider_name": null, "source_document_id": id(4), "field_key": "home.availability",
        "verification_state": "PROBABLE", "source_tier": "tier_2", "source_title": "source",
        "source_domain": "source.example", "published_at": null,
        "retrieved_at": "2026-10-13T00:00:00Z", "data_cutoff_at": "2026-10-13T01:00:00Z",
        "timeliness_score": 0.9, "decision_id": null, "verdict": null, "accuracy_score": null,
        "reliability_score": null, "decided_by": null, "decision_note": null, "decided_at": null,
        "status": "pending", "created_at": "2026-10-13T01:00:00Z"
    }))
    .unwrap()
}

struct Fixture {
    calls: Mutex<Vec<&'static str>>,
    query: Mutex<Option<(Option<String>, u32)>>,
    draft: Mutex<Option<EvidenceScoringDecisionDraft>>,
    error: Option<PortError>,
}
impl Fixture {
    fn new() -> Self {
        Self {
            calls: Mutex::new(Vec::new()),
            query: Mutex::new(None),
            draft: Mutex::new(None),
            error: None,
        }
    }
    fn call(&self, name: &'static str) -> PortResult<()> {
        self.calls.lock().unwrap().push(name);
        if let Some(error) = &self.error {
            return Err(error.clone());
        }
        Ok(())
    }
}

#[async_trait]
impl PostmatchSettlementPort for Fixture {
    async fn readiness(&self, _: Uuid) -> PortResult<PostmatchSettlementReadiness> {
        panic!("unexpected readiness")
    }
    async fn settle(&self, _: &PostmatchSettlementDraft) -> PortResult<PostmatchSettlementRecord> {
        panic!("unexpected settlement")
    }
    async fn list_settlements(&self, _: u32) -> PortResult<Vec<PostmatchSettlementRecord>> {
        panic!("unexpected settlements")
    }
    async fn list_evidence_scoring_items(
        &self,
        status: Option<&str>,
        limit: u32,
    ) -> PortResult<Vec<EvidenceScoringItemRecord>> {
        self.call("list")?;
        *self.query.lock().unwrap() = Some((status.map(str::to_owned), limit));
        Ok(vec![item_record()])
    }
    async fn decide_evidence_scoring_item(
        &self,
        draft: &EvidenceScoringDecisionDraft,
    ) -> PortResult<EvidenceScoringItemRecord> {
        self.call("decide")?;
        *self.draft.lock().unwrap() = Some(draft.clone());
        Ok(item_record())
    }
}

#[tokio::test]
async fn list_none_filter_preserves_zero_limit_and_full_port_records() {
    let fixture = Fixture::new();
    let result = list_evidence_scoring_items(&fixture, None, 0)
        .await
        .unwrap();
    assert_eq!(*fixture.query.lock().unwrap(), Some((None, 0)));
    assert_eq!(
        serde_json::to_value(result).unwrap(),
        serde_json::to_value(vec![item_record()]).unwrap()
    );
    assert_eq!(*fixture.calls.lock().unwrap(), ["list"]);
}

#[tokio::test]
async fn list_preserves_exact_optional_status_and_unclamped_limit() {
    let fixture = Fixture::new();
    for status in ["pending", "not_verifiable", " Pending ", "", "未知"] {
        list_evidence_scoring_items(&fixture, Some(status.into()), u32::MAX)
            .await
            .unwrap();
        assert_eq!(
            *fixture.query.lock().unwrap(),
            Some((Some(status.into()), u32::MAX))
        );
    }
    assert_eq!(*fixture.calls.lock().unwrap(), ["list"; 5]);
}

#[tokio::test]
async fn decision_preserves_all_four_verdicts_and_complete_returned_record() {
    let fixture = Fixture::new();
    for verdict in [
        EvidenceVerdict::Correct,
        EvidenceVerdict::Partial,
        EvidenceVerdict::Incorrect,
        EvidenceVerdict::NotVerifiable,
    ] {
        let draft = EvidenceScoringDecisionDraft {
            item_id: id(99),
            verdict,
            decided_by: Some(" reviewer ".into()),
            decision_note: "  原人工判定说明  ".into(),
        };
        let result = decide_evidence_scoring_item(&fixture, draft.clone())
            .await
            .unwrap();
        assert_eq!(
            serde_json::to_value(fixture.draft.lock().unwrap().as_ref().unwrap()).unwrap(),
            serde_json::to_value(draft).unwrap()
        );
        assert_eq!(
            serde_json::to_value(result).unwrap(),
            serde_json::to_value(item_record()).unwrap()
        );
    }
    assert_eq!(*fixture.calls.lock().unwrap(), ["decide"; 4]);
}

#[tokio::test]
async fn decision_keeps_missing_or_blank_actor_and_note_validation_at_port_boundary() {
    let fixture = Fixture::new();
    for actor in [None, Some("".into()), Some("\u{3000}审核员\u{3000}".into())] {
        let draft = EvidenceScoringDecisionDraft {
            item_id: id(1),
            verdict: EvidenceVerdict::Partial,
            decided_by: actor,
            decision_note: " 短 ".into(),
        };
        decide_evidence_scoring_item(&fixture, draft.clone())
            .await
            .unwrap();
        assert_eq!(
            serde_json::to_value(fixture.draft.lock().unwrap().as_ref().unwrap()).unwrap(),
            serde_json::to_value(draft).unwrap()
        );
    }
    assert_eq!(*fixture.calls.lock().unwrap(), ["decide"; 3]);
}

#[tokio::test]
async fn list_propagates_all_port_error_kinds_without_other_calls() {
    for kind in [
        PortErrorKind::Unavailable,
        PortErrorKind::NotFound,
        PortErrorKind::Conflict,
        PortErrorKind::InvalidState,
        PortErrorKind::Serialization,
        PortErrorKind::Infrastructure,
    ] {
        let mut fixture = Fixture::new();
        fixture.error = Some(PortError::new(kind, "original list failure"));
        assert!(
            matches!(list_evidence_scoring_items(&fixture, None, 0).await.unwrap_err(), ApplicationError::Port(error) if error.kind == kind && error.message == "original list failure")
        );
        assert_eq!(*fixture.calls.lock().unwrap(), ["list"]);
        assert!(fixture.draft.lock().unwrap().is_none());
    }
}

#[tokio::test]
async fn decision_propagates_port_failure_without_retry_or_monitoring() {
    let mut fixture = Fixture::new();
    fixture.error = Some(PortError::new(
        PortErrorKind::Conflict,
        "original decision failure",
    ));
    let draft = EvidenceScoringDecisionDraft {
        item_id: id(1),
        verdict: EvidenceVerdict::Correct,
        decided_by: None,
        decision_note: "original note".into(),
    };
    assert!(
        matches!(decide_evidence_scoring_item(&fixture, draft).await.unwrap_err(), ApplicationError::Port(error) if error.kind == PortErrorKind::Conflict && error.message == "original decision failure")
    );
    assert_eq!(*fixture.calls.lock().unwrap(), ["decide"]);
    assert!(fixture.query.lock().unwrap().is_none());
}
