use super::{confirmation::apply_confirmation_metadata, coordinator, guards};
use crate::{
    ports::{
        review::{
            MatchReviewPackageSourcePort, MatchReviewPackageStatePort,
            MatchReviewPackageValidationContext, MatchReviewPort,
        },
        PortError, PortErrorKind, PortResult,
    },
    ApplicationError,
};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use football_domain::{
    AbilityCandidateDecisionDraft, AbilityCandidateStatus, AbilityUpdateCandidateRecord,
    MatchEventSummary, MatchReviewDetail, MatchReviewDraft, MatchReviewPackageConfirmationRequest,
    MatchReviewPackageData, MatchReviewPackagePreview, MatchReviewPackageSummary,
    MatchReviewPackageWorkflowRecord, MatchReviewPackageWorkflowStatus, MatchReviewSummary,
    ReviewableMatch,
};
use serde_json::{json, Value};
use std::sync::Mutex;
use uuid::Uuid;

fn id(value: u128) -> Uuid {
    Uuid::from_u128(value)
}

fn preview() -> MatchReviewPackagePreview {
    let lineup = |team_id| {
        json!({
            "match_id": id(2), "team_id": id(team_id), "lineup_type": "actual",
            "captured_at": "2026-10-12T00:00:00Z", "players": [],
            "metadata": {"existing": true}
        })
    };
    serde_json::from_value(json!({
        "source_path": "review.xlsx", "source_file_name": "review.xlsx",
        "source_sha256": "sha256", "format_version": "football.match-review-package.v1",
        "package_id": id(1), "match_id": id(2), "match_key": "fixture",
        "home_team_name": "home", "away_team_name": "away",
        "lineup_pair": {"home": lineup(3), "away": lineup(4)},
        "review": {"match_id": id(2), "result": {
            "match_id": id(2), "home_goals_90": 1, "away_goals_90": 0,
            "finalized_at": "2026-10-12T00:00:00Z", "metadata": {"existing": true}
        }, "player_observations": [], "notes": " existing note "},
        "diff": {}, "home_player_count": 0, "away_player_count": 0,
        "home_starter_count": 0, "away_starter_count": 0,
        "substitution_count": 0, "observation_count": 0, "ready": true
    }))
    .unwrap()
}

fn workflow(status: MatchReviewPackageWorkflowStatus) -> MatchReviewPackageWorkflowRecord {
    let mut value: MatchReviewPackageWorkflowRecord = serde_json::from_value(json!({
        "package_id": id(1), "match_id": id(2), "match_key": "fixture",
        "status": "exported", "export_path": "export.xlsx", "export_sha256": "export-sha",
        "pre_match_snapshot": {}, "export_database_snapshot": {},
        "import_path": "review.xlsx", "import_sha256": "sha256", "preview_ready": true,
        "confirmed_by": "reviewer", "confirmation_note": " checked ",
        "exported_at": "2026-10-12T00:00:00Z", "confirmed_at": "2026-10-12T01:00:00Z",
        "updated_at": "2026-10-12T01:00:00Z"
    }))
    .unwrap();
    value.status = status;
    value.with_capabilities()
}

fn validation<T: std::fmt::Debug>(result: Result<T, ApplicationError>, expected: &str) {
    match result.unwrap_err() {
        ApplicationError::Validation(message) => assert_eq!(message, expected),
        other => panic!("expected validation, got {other:?}"),
    }
}

#[test]
fn revalidated_preview_checks_each_identity_and_blocker() {
    let mut state = workflow(MatchReviewPackageWorkflowStatus::PreviewValid);
    let original = preview();
    assert!(guards::matches_revalidated_preview(&state, &original));
    for mutation in 0..4 {
        let mut changed = original.clone();
        match mutation {
            0 => changed.package_id = id(99),
            1 => changed.source_sha256 = "changed".into(),
            2 => changed.ready = false,
            _ => changed.errors.push("blocked".into()),
        }
        assert!(!guards::matches_revalidated_preview(&state, &changed));
    }
    // Preserve the revalidation path's original None/empty-string compatibility.
    state.import_sha256 = None;
    let mut empty = original;
    empty.source_sha256.clear();
    assert!(guards::matches_revalidated_preview(&state, &empty));
}

#[test]
fn combined_commit_preserves_readiness_and_strict_identity_guards() {
    let mut state = workflow(MatchReviewPackageWorkflowStatus::PreviewValid);
    let original = preview();
    assert!(guards::ensure_commit_preview_ready(&original).is_ok());
    assert!(guards::ensure_commit_identity(&state, &original).is_ok());
    for mutation in 0..2 {
        let mut changed = original.clone();
        if mutation == 0 {
            changed.ready = false;
        } else {
            changed.errors.push("blocked".into());
        }
        validation(
            guards::ensure_commit_preview_ready(&changed),
            "赛后复盘资料包仍有阻断错误，不能确认导入",
        );
    }
    for mutation in 0..3 {
        let mut changed = original.clone();
        let mut changed_state = state.clone();
        match mutation {
            0 => changed.source_sha256 = "changed".into(),
            1 => changed.match_id = id(99),
            _ => changed_state = workflow(MatchReviewPackageWorkflowStatus::Confirmed),
        }
        validation(
            guards::ensure_commit_identity(&changed_state, &changed),
            "当前预检不属于本轮导出的资料包，请重新导出并预检",
        );
    }
    state.import_sha256 = None;
    let mut empty = original;
    empty.source_sha256.clear();
    validation(
        guards::ensure_commit_identity(&state, &empty),
        "当前预检不属于本轮导出的资料包，请重新导出并预检",
    );
}

#[test]
fn confirmation_metadata_preserves_existing_values_and_normalizes_non_objects() {
    let mut imported = preview();
    imported.lineup_pair.away.metadata = json!([1]);
    let state = workflow(MatchReviewPackageWorkflowStatus::Confirmed);
    apply_confirmation_metadata(&mut imported, &state);
    let expected = json!({"package_id": id(1), "source_sha256": "sha256",
        "confirmed_by": "reviewer", "confirmation_note": " checked ",
        "confirmed_at": "2026-10-12T01:00:00Z"});
    for metadata in [
        &imported.review.result.metadata,
        &imported.lineup_pair.home.metadata,
        &imported.lineup_pair.away.metadata,
    ] {
        assert_eq!(metadata["review_package_confirmation"], expected);
    }
    assert_eq!(imported.review.result.metadata["existing"], true);
    assert_eq!(imported.lineup_pair.home.metadata["existing"], true);
    assert_eq!(
        imported.review.notes.as_deref(),
        Some("existing note\n人工确认：checked")
    );
}

#[test]
fn confirmation_blank_note_preserves_notes_and_empty_existing_note_is_replaced() {
    for note in [None, Some("   ".to_string())] {
        let mut imported = preview();
        let mut state = workflow(MatchReviewPackageWorkflowStatus::Confirmed);
        state.confirmation_note = note;
        apply_confirmation_metadata(&mut imported, &state);
        assert_eq!(imported.review.notes.as_deref(), Some(" existing note "));
    }
    let mut imported = preview();
    imported.review.notes = Some("  ".into());
    apply_confirmation_metadata(
        &mut imported,
        &workflow(MatchReviewPackageWorkflowStatus::Confirmed),
    );
    assert_eq!(imported.review.notes.as_deref(), Some("人工确认：checked"));
}

struct WorkflowFixture {
    workflow: MatchReviewPackageWorkflowRecord,
    preview: MatchReviewPackagePreview,
    calls: Mutex<Vec<&'static str>>,
    generated_draft: Mutex<Option<MatchReviewDraft>>,
    fail_at: Option<&'static str>,
    active: bool,
}

impl WorkflowFixture {
    fn new(status: MatchReviewPackageWorkflowStatus) -> Self {
        Self {
            workflow: workflow(status),
            preview: preview(),
            calls: Mutex::new(Vec::new()),
            generated_draft: Mutex::new(None),
            fail_at: None,
            active: true,
        }
    }

    fn record(&self, call: &'static str) -> PortResult<()> {
        self.calls.lock().unwrap().push(call);
        if self.fail_at == Some(call) {
            Err(PortError::new(PortErrorKind::Infrastructure, call))
        } else {
            Ok(())
        }
    }
}

fn unused<T>() -> PortResult<T> {
    panic!("unexpected workflow dependency")
}

#[async_trait]
impl MatchReviewPackageStatePort for WorkflowFixture {
    async fn register_export(
        &self,
        _: &MatchReviewPackageSummary,
    ) -> PortResult<MatchReviewPackageWorkflowRecord> {
        unused()
    }
    async fn read_active_workflow(
        &self,
        match_id: Uuid,
    ) -> PortResult<Option<MatchReviewPackageWorkflowRecord>> {
        assert_eq!(match_id, id(2));
        self.record("read_active_workflow")?;
        Ok(self.active.then(|| self.workflow.clone()))
    }
    async fn read_workflow(
        &self,
        package_id: Uuid,
    ) -> PortResult<MatchReviewPackageWorkflowRecord> {
        assert_eq!(package_id, id(1));
        self.record("read_workflow")?;
        Ok(self.workflow.clone())
    }
    async fn read_workflow_by_review(
        &self,
        _: Uuid,
    ) -> PortResult<Option<MatchReviewPackageWorkflowRecord>> {
        unused()
    }
    async fn mark_settled(&self, _: Uuid) -> PortResult<Option<MatchReviewPackageWorkflowRecord>> {
        unused()
    }
    async fn record_preview(
        &self,
        _: Uuid,
        _: &MatchReviewPackagePreview,
    ) -> PortResult<MatchReviewPackageWorkflowRecord> {
        unused()
    }
    async fn confirm_workflow(
        &self,
        _: Uuid,
        _: Option<&str>,
        _: Option<&str>,
    ) -> PortResult<MatchReviewPackageWorkflowRecord> {
        unused()
    }
    async fn mark_facts_committed(&self, _: Uuid) -> PortResult<MatchReviewPackageWorkflowRecord> {
        unused()
    }
    async fn mark_review_created(
        &self,
        package_id: Uuid,
        review_id: Uuid,
    ) -> PortResult<MatchReviewPackageWorkflowRecord> {
        assert_eq!(package_id, id(1));
        assert_eq!(review_id, id(5));
        self.record("mark_review_created")?;
        let mut state = self.workflow.clone();
        state.status = MatchReviewPackageWorkflowStatus::ReviewCreated;
        state.review_id = Some(review_id);
        Ok(state.with_capabilities())
    }
    async fn read_package_preview(
        &self,
        package_id: Uuid,
    ) -> PortResult<MatchReviewPackagePreview> {
        assert_eq!(package_id, id(1));
        self.record("read_package_preview")?;
        Ok(self.preview.clone())
    }
}

#[async_trait]
impl MatchReviewPackageSourcePort for WorkflowFixture {
    async fn build_export_data(
        &self,
        _: Uuid,
        _: Uuid,
        _: DateTime<Utc>,
    ) -> PortResult<MatchReviewPackageData> {
        unused()
    }
    async fn validation_context(&self, _: Uuid) -> PortResult<MatchReviewPackageValidationContext> {
        unused()
    }
    async fn read_source_run(&self, _: Uuid) -> PortResult<Value> {
        unused()
    }
}

#[async_trait]
impl MatchReviewPort for WorkflowFixture {
    async fn list_reviewable_matches(&self, _: u32) -> PortResult<Vec<ReviewableMatch>> {
        unused()
    }
    async fn generate_match_review(
        &self,
        draft: &MatchReviewDraft,
    ) -> PortResult<MatchReviewDetail> {
        self.record("generate_match_review")?;
        *self.generated_draft.lock().unwrap() = Some(draft.clone());
        Ok(serde_json::from_value(json!({
            "summary": {"id": id(5), "match_id": id(2), "match_key": "fixture",
                "home_team_name": "home", "away_team_name": "away", "review_version": "v1",
                "status": "created", "data_coverage": 1.0, "calculation_version": "fixture",
                "result_snapshot": {}, "substitutions_snapshot": [], "prediction_evaluation": {},
                "conclusions": {}, "created_at": "2026-10-12T02:00:00Z"},
            "result": draft.result, "substitutions": [], "events": [],
            "event_summary": MatchEventSummary::default(), "player_reviews": [],
            "team_reviews": [], "ability_candidates": []
        }))
        .unwrap())
    }
    async fn list_match_reviews(&self, _: u32) -> PortResult<Vec<MatchReviewSummary>> {
        unused()
    }
    async fn read_match_review(&self, _: Uuid) -> PortResult<MatchReviewDetail> {
        unused()
    }
    async fn list_ability_candidates(
        &self,
        _: Option<AbilityCandidateStatus>,
        _: u32,
        _: Option<Uuid>,
    ) -> PortResult<Vec<AbilityUpdateCandidateRecord>> {
        unused()
    }
    async fn decide_ability_candidate(
        &self,
        _: &AbilityCandidateDecisionDraft,
    ) -> PortResult<AbilityUpdateCandidateRecord> {
        unused()
    }
}

#[tokio::test]
async fn confirmation_rejects_invalid_state_readiness_and_missing_path_before_io() {
    for case in 0..3 {
        let mut port = WorkflowFixture::new(MatchReviewPackageWorkflowStatus::PreviewValid);
        let expected = match case {
            0 => {
                port.workflow = workflow(MatchReviewPackageWorkflowStatus::Exported);
                port.workflow
                    .require_action(
                        football_domain::MatchReviewPackageWorkflowAction::ConfirmImport,
                    )
                    .unwrap_err()
            }
            1 => {
                port.workflow.preview_ready = false;
                "本轮导出的资料包尚未通过预检，不能人工确认".into()
            }
            _ => {
                port.workflow.import_path = None;
                "预检记录缺少导入文件路径，请重新预检".into()
            }
        };
        validation(
            coordinator::confirm(
                &port,
                MatchReviewPackageConfirmationRequest {
                    package_id: id(1),
                    confirmed_by: None,
                    confirmation_note: None,
                },
            )
            .await,
            &expected,
        );
        assert_eq!(*port.calls.lock().unwrap(), ["read_workflow"]);
    }
}

#[tokio::test]
async fn generate_review_preserves_order_confirmation_and_returned_ids() {
    let port = WorkflowFixture::new(MatchReviewPackageWorkflowStatus::FactsCommitted);
    let result = coordinator::generate_review(&port, id(1)).await.unwrap();
    assert_eq!(
        *port.calls.lock().unwrap(),
        [
            "read_workflow",
            "read_package_preview",
            "generate_match_review",
            "mark_review_created"
        ]
    );
    assert_eq!(result.review.summary.id, id(5));
    assert_eq!(result.workflow.review_id, Some(id(5)));
    assert_eq!(
        result.workflow.status,
        MatchReviewPackageWorkflowStatus::ReviewCreated
    );
    let captured = port.generated_draft.lock().unwrap();
    let draft = captured.as_ref().unwrap();
    assert_eq!(
        draft.result.metadata["review_package_confirmation"]["package_id"],
        json!(id(1))
    );
    assert_eq!(
        draft.notes.as_deref(),
        Some("existing note\n人工确认：checked")
    );
}

#[tokio::test]
async fn generate_review_stops_at_failed_dependency_or_domain_guard() {
    let calls = [
        "read_workflow",
        "read_package_preview",
        "generate_match_review",
        "mark_review_created",
    ];
    for (index, call) in calls.iter().enumerate() {
        let mut port = WorkflowFixture::new(MatchReviewPackageWorkflowStatus::FactsCommitted);
        port.fail_at = Some(*call);
        let error = coordinator::generate_review(&port, id(1))
            .await
            .unwrap_err();
        assert!(error.to_string().contains(*call));
        assert_eq!(*port.calls.lock().unwrap(), calls[..=index]);
    }
    let port = WorkflowFixture::new(MatchReviewPackageWorkflowStatus::Confirmed);
    assert!(coordinator::generate_review(&port, id(1)).await.is_err());
    assert_eq!(*port.calls.lock().unwrap(), ["read_workflow"]);
}

#[tokio::test]
async fn active_workflow_preserves_some_none_and_port_failure_without_writes() {
    for active in [true, false] {
        let mut port = WorkflowFixture::new(MatchReviewPackageWorkflowStatus::Exported);
        port.active = active;
        let result = coordinator::read_active_workflow(&port, id(2))
            .await
            .unwrap();
        assert_eq!(
            result.map(|record| record.package_id),
            active.then(|| id(1))
        );
        assert_eq!(*port.calls.lock().unwrap(), ["read_active_workflow"]);
    }
    let mut port = WorkflowFixture::new(MatchReviewPackageWorkflowStatus::Exported);
    port.fail_at = Some("read_active_workflow");
    assert!(coordinator::read_active_workflow(&port, id(2))
        .await
        .is_err());
    assert_eq!(*port.calls.lock().unwrap(), ["read_active_workflow"]);
}
