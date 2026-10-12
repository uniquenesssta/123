use super::{
    mapping::candidate_status_from_db,
    validation::{decision_actor, require_current_projection, require_pending_candidate},
};
use crate::PersistenceError;
use football_domain::AbilityCandidateStatus;

#[test]
fn pending_guard_rejects_every_decided_status() {
    assert!(require_pending_candidate("pending").is_ok());
    for status in ["accepted", "rejected", "superseded"] {
        assert!(matches!(
            require_pending_candidate(status),
            Err(PersistenceError::InvalidState(message))
                if message == format!("能力更新候选已经处理，当前状态：{status}")
        ));
    }
}

#[test]
fn pending_guard_preserves_exact_status_and_error_text() {
    for status in ["", "Pending", " pending", "pending ", "未知状态"] {
        assert!(matches!(
            require_pending_candidate(status),
            Err(PersistenceError::InvalidState(message))
                if message == format!("能力更新候选已经处理，当前状态：{status}")
        ));
    }
}

#[test]
fn decision_actor_defaults_only_for_missing_or_blank_identity() {
    for actor in [None, Some(""), Some(" \t\n"), Some("\u{3000}\u{2003}")] {
        assert_eq!(decision_actor(actor), "local_user");
    }
}

#[test]
fn decision_actor_trims_unicode_without_changing_identity() {
    assert_eq!(
        decision_actor(Some("\u{3000}人工审核员\u{2003}")),
        "人工审核员"
    );
    assert_eq!(decision_actor(Some("  Alice Reviewer  ")), "Alice Reviewer");
    assert_eq!(decision_actor(Some("0")), "0");
}

#[test]
fn current_projection_accepts_missing_pair_and_inclusive_tolerance() {
    assert!(require_current_projection(None, None).is_ok());
    for (expected, actual) in [(50.0, 50.0), (0.0, 0.01), (0.0, -0.01), (50.0, 50.009)] {
        assert!(require_current_projection(Some(expected), Some(actual)).is_ok());
    }
}

#[test]
fn current_projection_rejects_stale_missing_and_nonfinite_values() {
    for (expected, actual) in [
        (Some(50.0), None),
        (None, Some(50.0)),
        (Some(0.0), Some(0.010_001)),
        (Some(0.0), Some(-0.010_001)),
        (Some(f64::NAN), Some(50.0)),
        (Some(50.0), Some(f64::NAN)),
        (Some(f64::INFINITY), Some(f64::INFINITY)),
        (Some(f64::NEG_INFINITY), Some(0.0)),
    ] {
        assert!(matches!(
            require_current_projection(expected, actual),
            Err(PersistenceError::InvalidState(message))
                if message == "球员当前能力已发生变化，该候选已经过期，请重新生成复盘"
        ));
    }
}

#[test]
fn candidate_status_mapping_preserves_four_domain_states() {
    for status in [
        AbilityCandidateStatus::Pending,
        AbilityCandidateStatus::Accepted,
        AbilityCandidateStatus::Rejected,
        AbilityCandidateStatus::Superseded,
    ] {
        assert_eq!(candidate_status_from_db(status.as_str()).unwrap(), status);
    }
}

#[test]
fn candidate_status_mapping_rejects_unknown_without_normalization() {
    for status in ["", "Pending", " pending", "accepted ", "未知状态"] {
        assert!(matches!(
            candidate_status_from_db(status),
            Err(PersistenceError::InvalidState(message))
                if message == format!("未知候选状态：{status}")
        ));
    }
}
