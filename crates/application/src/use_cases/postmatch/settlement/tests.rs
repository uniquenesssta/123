use super::{list_postmatch_settlements, postmatch_settlement_readiness, settle_postmatch_review};
use crate::{
    ports::{
        postmatch::PostmatchSettlementPort, review::MatchReviewPackageStatePort, PortError,
        PortErrorKind, PortResult,
    },
    ApplicationError,
};
use async_trait::async_trait;
use football_domain::{
    EvidenceScoringDecisionDraft, EvidenceScoringItemRecord, MatchReviewPackagePreview,
    MatchReviewPackageSummary, MatchReviewPackageWorkflowAction, MatchReviewPackageWorkflowRecord,
    MatchReviewPackageWorkflowStatus, PostmatchSettlementDraft, PostmatchSettlementReadiness,
    PostmatchSettlementRecord,
};
use serde_json::json;
use std::sync::Mutex;
use uuid::Uuid;

fn id(value: u128) -> Uuid {
    Uuid::from_u128(value)
}

fn workflow(status: MatchReviewPackageWorkflowStatus) -> MatchReviewPackageWorkflowRecord {
    let mut record: MatchReviewPackageWorkflowRecord = serde_json::from_value(json!({
        "package_id": id(3), "match_id": id(4), "match_key": "fixture", "status": "exported",
        "export_path": "review.xlsx", "export_sha256": "sha256", "pre_match_snapshot": {},
        "export_database_snapshot": {}, "preview_ready": true,
        "exported_at": "2026-10-13T00:00:00Z", "updated_at": "2026-10-13T00:00:00Z"
    }))
    .unwrap();
    record.status = status;
    record.with_capabilities()
}

fn settlement() -> PostmatchSettlementRecord {
    serde_json::from_value(json!({
        "id": id(5), "match_id": id(4), "match_review_id": id(2), "model_run_id": id(6),
        "feature_snapshot_id": id(7), "competition_id": id(8), "competition_name": "competition",
        "competition_profile_id": id(9), "model_version_id": id(10), "model_version": "version",
        "parameter_set_id": id(11), "parameter_version": "parameter", "rule_package_id": id(12),
        "horizon": "T-6h", "match_key": "fixture", "home_team_name": "home", "away_team_name": "away",
        "home_goals_90": 2, "away_goals_90": 1, "result_finalized_at": "2026-10-13T00:00:00Z",
        "result_fingerprint": "original-fingerprint", "settlement_key": "original-key",
        "settlement_version": "postmatch-settlement-v1", "status": "settled",
        "evidence_item_count": 7, "scored_evidence_count": 2, "drift_status": "unavailable",
        "settled_by": " reviewed ", "settlement_note": " note ",
        "metadata": {"automatic_parameter_promotion": false}, "settled_at": "2026-10-13T00:00:00Z"
    })).unwrap()
}

fn readiness() -> PostmatchSettlementReadiness {
    serde_json::from_value(json!({
        "match_review_id": id(1), "match_id": id(4), "match_key": "fixture",
        "home_team_name": "home", "away_team_name": "away", "result_ready": false,
        "finalized_review_ready": true, "successful_run_ready": true, "frozen_snapshot_ready": true,
        "snapshot_identity_ready": true, "real_evidence_snapshot_ready": true,
        "competition_profile_ready": true, "formal_horizon_ready": true,
        "existing_settlement_id": id(5), "blocked_reasons": ["original blocker"], "ready": false
    }))
    .unwrap()
}

fn draft() -> PostmatchSettlementDraft {
    PostmatchSettlementDraft {
        match_review_id: id(1),
        settled_by: Some(" reviewer ".into()),
        settlement_note: Some(" note ".into()),
    }
}

struct Fixture {
    workflow: Option<MatchReviewPackageWorkflowRecord>,
    calls: Mutex<Vec<&'static str>>,
    captured: Mutex<Option<PostmatchSettlementDraft>>,
    marked: Mutex<Vec<Uuid>>,
    readiness_id: Mutex<Option<Uuid>>,
    list_limit: Mutex<Option<u32>>,
    fail_at: Option<&'static str>,
}
impl Fixture {
    fn new(status: Option<MatchReviewPackageWorkflowStatus>) -> Self {
        Self {
            workflow: status.map(workflow),
            calls: Mutex::new(Vec::new()),
            captured: Mutex::new(None),
            marked: Mutex::new(Vec::new()),
            readiness_id: Mutex::new(None),
            list_limit: Mutex::new(None),
            fail_at: None,
        }
    }
    fn call(&self, name: &'static str) -> PortResult<()> {
        self.calls.lock().unwrap().push(name);
        if self.fail_at == Some(name) {
            return Err(PortError::new(PortErrorKind::Conflict, name));
        }
        Ok(())
    }
}

#[async_trait]
impl PostmatchSettlementPort for Fixture {
    async fn readiness(&self, review_id: Uuid) -> PortResult<PostmatchSettlementReadiness> {
        self.call("readiness")?;
        *self.readiness_id.lock().unwrap() = Some(review_id);
        Ok(readiness())
    }
    async fn settle(
        &self,
        draft: &PostmatchSettlementDraft,
    ) -> PortResult<PostmatchSettlementRecord> {
        self.call("settle")?;
        *self.captured.lock().unwrap() = Some(draft.clone());
        Ok(settlement())
    }
    async fn list_settlements(&self, limit: u32) -> PortResult<Vec<PostmatchSettlementRecord>> {
        self.call("list")?;
        *self.list_limit.lock().unwrap() = Some(limit);
        Ok(vec![settlement()])
    }
    async fn list_evidence_scoring_items(
        &self,
        _: Option<&str>,
        _: u32,
    ) -> PortResult<Vec<EvidenceScoringItemRecord>> {
        panic!("unexpected evidence read")
    }
    async fn decide_evidence_scoring_item(
        &self,
        _: &EvidenceScoringDecisionDraft,
    ) -> PortResult<EvidenceScoringItemRecord> {
        panic!("unexpected evidence decision")
    }
}

#[async_trait]
impl MatchReviewPackageStatePort for Fixture {
    async fn read_workflow_by_review(
        &self,
        review_id: Uuid,
    ) -> PortResult<Option<MatchReviewPackageWorkflowRecord>> {
        assert_eq!(review_id, id(1));
        self.call("workflow")?;
        Ok(self.workflow.clone())
    }
    async fn mark_settled(
        &self,
        review_id: Uuid,
    ) -> PortResult<Option<MatchReviewPackageWorkflowRecord>> {
        self.call("mark")?;
        self.marked.lock().unwrap().push(review_id);
        Ok(self.workflow.clone())
    }
    async fn register_export(
        &self,
        _: &MatchReviewPackageSummary,
    ) -> PortResult<MatchReviewPackageWorkflowRecord> {
        panic!("unexpected export")
    }
    async fn read_active_workflow(
        &self,
        _: Uuid,
    ) -> PortResult<Option<MatchReviewPackageWorkflowRecord>> {
        panic!("unexpected active read")
    }
    async fn read_workflow(&self, _: Uuid) -> PortResult<MatchReviewPackageWorkflowRecord> {
        panic!("unexpected workflow read")
    }
    async fn record_preview(
        &self,
        _: Uuid,
        _: &MatchReviewPackagePreview,
    ) -> PortResult<MatchReviewPackageWorkflowRecord> {
        panic!("unexpected preview")
    }
    async fn confirm_workflow(
        &self,
        _: Uuid,
        _: Option<&str>,
        _: Option<&str>,
    ) -> PortResult<MatchReviewPackageWorkflowRecord> {
        panic!("unexpected confirmation")
    }
    async fn mark_facts_committed(&self, _: Uuid) -> PortResult<MatchReviewPackageWorkflowRecord> {
        panic!("unexpected facts")
    }
    async fn mark_review_created(
        &self,
        _: Uuid,
        _: Uuid,
    ) -> PortResult<MatchReviewPackageWorkflowRecord> {
        panic!("unexpected generation")
    }
    async fn read_package_preview(&self, _: Uuid) -> PortResult<MatchReviewPackagePreview> {
        panic!("unexpected preview read")
    }
}

fn assert_port_error(result: Result<impl std::fmt::Debug, ApplicationError>, name: &str) {
    assert!(
        matches!(result.unwrap_err(), ApplicationError::Port(error) if error.kind == PortErrorKind::Conflict && error.message == name)
    );
}

#[tokio::test]
async fn plain_settlement_preserves_draft_detail_and_returned_review_identity() {
    let fixture = Fixture::new(None);
    let request = draft();
    let result = settle_postmatch_review(&fixture, request.clone())
        .await
        .unwrap();
    assert_eq!(
        serde_json::to_value(result).unwrap(),
        serde_json::to_value(settlement()).unwrap()
    );
    assert_eq!(
        serde_json::to_value(fixture.captured.lock().unwrap().as_ref().unwrap()).unwrap(),
        serde_json::to_value(request).unwrap()
    );
    assert_eq!(
        *fixture.calls.lock().unwrap(),
        ["workflow", "settle", "mark"]
    );
    assert_eq!(*fixture.marked.lock().unwrap(), [id(2)]);
}

#[tokio::test]
async fn review_created_package_settles_then_marks_once() {
    let fixture = Fixture::new(Some(MatchReviewPackageWorkflowStatus::ReviewCreated));
    settle_postmatch_review(&fixture, draft()).await.unwrap();
    assert_eq!(
        *fixture.calls.lock().unwrap(),
        ["workflow", "settle", "mark"]
    );
    assert_eq!(*fixture.marked.lock().unwrap(), [id(2)]);
}

#[tokio::test]
async fn settled_package_retries_original_idempotent_port_without_new_readiness() {
    let fixture = Fixture::new(Some(MatchReviewPackageWorkflowStatus::Settled));
    for _ in 0..2 {
        assert_eq!(
            settle_postmatch_review(&fixture, draft()).await.unwrap().id,
            id(5)
        );
    }
    assert_eq!(
        *fixture.calls.lock().unwrap(),
        ["workflow", "settle", "mark", "workflow", "settle", "mark"]
    );
}

#[tokio::test]
async fn blocked_package_states_stop_before_settlement_with_original_domain_reason() {
    use MatchReviewPackageWorkflowStatus as S;
    for status in [
        S::Exported,
        S::PreviewBlocked,
        S::PreviewValid,
        S::Confirmed,
        S::FactsCommitted,
        S::Superseded,
    ] {
        let fixture = Fixture::new(Some(status));
        let expected = workflow(status)
            .require_action(MatchReviewPackageWorkflowAction::SettleReview)
            .unwrap_err();
        assert!(
            matches!(settle_postmatch_review(&fixture, draft()).await.unwrap_err(), ApplicationError::Validation(message) if message == expected)
        );
        assert_eq!(*fixture.calls.lock().unwrap(), ["workflow"]);
    }
}

#[tokio::test]
async fn workflow_lookup_failure_stops_all_settlement_effects() {
    let mut fixture = Fixture::new(None);
    fixture.fail_at = Some("workflow");
    assert_port_error(settle_postmatch_review(&fixture, draft()).await, "workflow");
    assert_eq!(*fixture.calls.lock().unwrap(), ["workflow"]);
    assert!(fixture.captured.lock().unwrap().is_none());
}

#[tokio::test]
async fn settlement_and_mark_errors_preserve_original_side_effect_boundary() {
    for (failed, calls) in [
        ("settle", vec!["workflow", "settle"]),
        ("mark", vec!["workflow", "settle", "mark"]),
    ] {
        let mut fixture = Fixture::new(None);
        fixture.fail_at = Some(failed);
        assert_port_error(settle_postmatch_review(&fixture, draft()).await, failed);
        assert_eq!(*fixture.calls.lock().unwrap(), calls);
        assert!(fixture.marked.lock().unwrap().is_empty());
    }
}

#[tokio::test]
async fn readiness_and_list_preserve_input_parameters_and_complete_results() {
    let fixture = Fixture::new(None);
    let result = postmatch_settlement_readiness(&fixture, id(99))
        .await
        .unwrap();
    assert_eq!(*fixture.readiness_id.lock().unwrap(), Some(id(99)));
    assert_eq!(
        serde_json::to_value(result).unwrap(),
        serde_json::to_value(readiness()).unwrap()
    );
    for limit in [0, u32::MAX] {
        let records = list_postmatch_settlements(&fixture, limit).await.unwrap();
        assert_eq!(*fixture.list_limit.lock().unwrap(), Some(limit));
        assert_eq!(
            serde_json::to_value(records).unwrap(),
            serde_json::to_value(vec![settlement()]).unwrap()
        );
    }
    assert_eq!(
        *fixture.calls.lock().unwrap(),
        ["readiness", "list", "list"]
    );
}

#[tokio::test]
async fn readiness_and_list_propagate_port_errors_without_workflow_writes() {
    for failed in ["readiness", "list"] {
        let mut fixture = Fixture::new(None);
        fixture.fail_at = Some(failed);
        if failed == "readiness" {
            assert_port_error(
                postmatch_settlement_readiness(&fixture, id(99)).await,
                failed,
            );
        } else {
            assert_port_error(list_postmatch_settlements(&fixture, 0).await, failed);
        }
        assert_eq!(*fixture.calls.lock().unwrap(), [failed]);
    }
}
