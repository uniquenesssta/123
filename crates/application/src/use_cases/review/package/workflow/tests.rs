use super::{confirmation::apply_confirmation_metadata, coordinator, guards};
use crate::use_cases::review::test_support::{id, preview, validation, workflow, WorkflowFixture};
use football_domain::{MatchReviewPackageConfirmationRequest, MatchReviewPackageWorkflowStatus};
use serde_json::json;

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
