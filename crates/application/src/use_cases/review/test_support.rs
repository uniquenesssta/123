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
    MatchEventSummary, MatchReviewDetail, MatchReviewDraft, MatchReviewPackageData,
    MatchReviewPackagePreview, MatchReviewPackageSummary, MatchReviewPackageWorkflowRecord,
    MatchReviewPackageWorkflowStatus, MatchReviewSummary, ReviewableMatch,
};
use serde_json::{json, Value};
use std::sync::Mutex;
use uuid::Uuid;

pub(super) fn id(value: u128) -> Uuid {
    Uuid::from_u128(value)
}

pub(super) fn preview() -> MatchReviewPackagePreview {
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

pub(super) fn workflow(
    status: MatchReviewPackageWorkflowStatus,
) -> MatchReviewPackageWorkflowRecord {
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

pub(super) fn validation<T: std::fmt::Debug>(result: Result<T, ApplicationError>, expected: &str) {
    match result.unwrap_err() {
        ApplicationError::Validation(message) => assert_eq!(message, expected),
        other => panic!("expected validation, got {other:?}"),
    }
}

pub(super) struct WorkflowFixture {
    pub(super) workflow: MatchReviewPackageWorkflowRecord,
    pub(super) preview: MatchReviewPackagePreview,
    pub(super) calls: Mutex<Vec<&'static str>>,
    pub(super) generated_draft: Mutex<Option<MatchReviewDraft>>,
    pub(super) fail_at: Option<&'static str>,
    pub(super) active: bool,
}

impl WorkflowFixture {
    pub(super) fn new(status: MatchReviewPackageWorkflowStatus) -> Self {
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
        Ok(generated_review(draft))
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

pub(super) fn generated_review(draft: &MatchReviewDraft) -> MatchReviewDetail {
    serde_json::from_value(json!({
        "summary": {"id": id(5), "match_id": id(2), "match_key": "fixture",
            "home_team_name": "home", "away_team_name": "away", "review_version": "v1",
            "status": "created", "data_coverage": 1.0, "calculation_version": "fixture",
            "result_snapshot": {}, "substitutions_snapshot": [], "prediction_evaluation": {},
            "conclusions": {}, "created_at": "2026-10-12T02:00:00Z"},
        "result": draft.result, "substitutions": [], "events": [],
        "event_summary": MatchEventSummary::default(), "player_reviews": [],
        "team_reviews": [], "ability_candidates": []
    }))
    .unwrap()
}
