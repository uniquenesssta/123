use super::{generate_match_review, generate_review};
use crate::use_cases::review::test_support::{
    generated_review, id, preview, validation, WorkflowFixture,
};
use football_domain::{MatchReviewDraft, MatchReviewPackageWorkflowStatus};
use serde_json::{json, Value};

#[tokio::test]
async fn generate_review_preserves_order_confirmation_and_returned_ids() {
    let port = WorkflowFixture::new(MatchReviewPackageWorkflowStatus::FactsCommitted);
    let result = generate_review(&port, id(1)).await.unwrap();
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
        let error = generate_review(&port, id(1)).await.unwrap_err();
        assert!(error.to_string().contains(*call));
        assert_eq!(*port.calls.lock().unwrap(), calls[..=index]);
    }
    let port = WorkflowFixture::new(MatchReviewPackageWorkflowStatus::Confirmed);
    assert!(generate_review(&port, id(1)).await.is_err());
    assert_eq!(*port.calls.lock().unwrap(), ["read_workflow"]);
}

fn draft() -> MatchReviewDraft {
    preview().review
}

#[tokio::test]
async fn draft_mismatched_result_stops_before_engine_port() {
    let port = WorkflowFixture::new(MatchReviewPackageWorkflowStatus::Exported);
    let mut request = draft();
    request.result.match_id = id(99);
    validation(
        generate_match_review(&port, request).await,
        "正式赛果与复盘比赛不一致",
    );
    assert!(port.calls.lock().unwrap().is_empty());
    assert!(port.generated_draft.lock().unwrap().is_none());
}

#[tokio::test]
async fn draft_blank_version_and_notes_preserve_optional_contract() {
    for version in [None, Some(String::new()), Some(" \t\n　".into())] {
        let port = WorkflowFixture::new(MatchReviewPackageWorkflowStatus::Exported);
        let mut request = draft();
        request.review_version = version;
        request.notes = Some(" \t\n　".into());
        generate_match_review(&port, request).await.unwrap();
        let captured = port.generated_draft.lock().unwrap();
        let captured = captured.as_ref().unwrap();
        assert_eq!(captured.review_version, None);
        assert_eq!(captured.notes.as_deref(), Some(""));
        assert_eq!(*port.calls.lock().unwrap(), ["generate_match_review"]);
    }
    let port = WorkflowFixture::new(MatchReviewPackageWorkflowStatus::Exported);
    let mut request = draft();
    request.notes = None;
    generate_match_review(&port, request).await.unwrap();
    assert_eq!(
        port.generated_draft.lock().unwrap().as_ref().unwrap().notes,
        None
    );
}

#[tokio::test]
async fn draft_normalization_preserves_fact_payload_and_source_context() {
    let port = WorkflowFixture::new(MatchReviewPackageWorkflowStatus::Exported);
    let mut request = draft();
    request.review_version = Some("　复盘 r1　".into());
    request.notes = Some("　人工记录　".into());
    request.data_coverage = 0.7;
    request.source_run_id = Some(id(8));
    request.result.metadata = json!({"source": "record", "revision": 1});
    request.events.push(
        serde_json::from_value(json!({"event_type": "goal", "minute": 12,
        "team_id": id(3), "player_id": id(6), "event_key": "goal:1", "sequence_no": 1}))
        .unwrap(),
    );
    let facts = json!({"result": request.result, "events": request.events,
        "substitutions": request.substitutions, "observations": request.player_observations});
    generate_match_review(&port, request).await.unwrap();
    let captured = port.generated_draft.lock().unwrap();
    let captured = captured.as_ref().unwrap();
    assert_eq!(captured.review_version.as_deref(), Some("复盘 r1"));
    assert_eq!(captured.notes.as_deref(), Some("人工记录"));
    assert_eq!(captured.data_coverage, 0.7);
    assert_eq!(captured.source_run_id, Some(id(8)));
    assert_eq!(
        json!({"result": captured.result, "events": captured.events,
        "substitutions": captured.substitutions, "observations": captured.player_observations}),
        facts
    );
    assert_eq!(*port.calls.lock().unwrap(), ["generate_match_review"]);
}

#[tokio::test]
async fn draft_generation_returns_the_complete_port_detail() {
    let port = WorkflowFixture::new(MatchReviewPackageWorkflowStatus::Exported);
    let request = draft();
    let expected: Value = serde_json::to_value(generated_review(&request)).unwrap();
    let result = generate_match_review(&port, request).await.unwrap();
    assert_eq!(serde_json::to_value(result).unwrap(), expected);
    assert_eq!(*port.calls.lock().unwrap(), ["generate_match_review"]);
}

#[tokio::test]
async fn draft_generation_propagates_port_failure_without_workflow_writes() {
    let mut port = WorkflowFixture::new(MatchReviewPackageWorkflowStatus::Exported);
    port.fail_at = Some("generate_match_review");
    let error = generate_match_review(&port, draft()).await.unwrap_err();
    assert!(error.to_string().contains("generate_match_review"));
    assert_eq!(*port.calls.lock().unwrap(), ["generate_match_review"]);
    assert!(port.generated_draft.lock().unwrap().is_none());
}
