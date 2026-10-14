use super::policy::*;
use crate::PersistenceError;
use football_domain::ParameterTuningDecision;
use uuid::Uuid;

#[test]
fn candidate_manual_decisions_keep_original_backtest_and_rejection_states() {
    assert_eq!(
        tuning_decision_status(ParameterTuningDecision::AcceptForBacktest),
        "accepted_for_backtest"
    );
    assert_eq!(
        tuning_decision_status(ParameterTuningDecision::Reject),
        "rejected"
    );
}

#[test]
fn shadow_result_status_keeps_exact_pass_fail_and_original_fallback() {
    assert_eq!(shadow_candidate_status("passed"), "shadow_passed");
    assert_eq!(shadow_candidate_status("failed"), "shadow_failed");
    for status in ["blocked", "", " passed", "Passed", "unknown"] {
        assert_eq!(shadow_candidate_status(status), "blocked_by_h");
    }
}

#[test]
fn promotion_accepts_only_exact_shadow_passed_candidate() {
    ensure_promotable("shadow_passed").unwrap();
    for status in [
        "pending",
        "accepted_for_backtest",
        "promoted",
        "shadow_passed ",
        "",
    ] {
        assert!(
            matches!(ensure_promotable(status).unwrap_err(), PersistenceError::InvalidState(message)
            if message == "只有影子验证通过的候选可以人工晋升")
        );
    }
}

#[test]
fn rollback_accepts_only_exact_promoted_candidate() {
    ensure_rollback_allowed("promoted").unwrap();
    for status in ["pending", "shadow_passed", "rolled_back", " promoted", ""] {
        assert!(
            matches!(ensure_rollback_allowed(status).unwrap_err(), PersistenceError::InvalidState(message)
            if message == "只有已晋升且未回滚的候选可以回滚")
        );
    }
}

#[test]
fn required_identity_preserves_nil_and_exact_missing_label() {
    for id in [Uuid::nil(), Uuid::from_u128(7)] {
        assert_eq!(required_uuid(Some(id), "unused").unwrap(), id);
    }
    assert!(
        matches!(required_uuid(None, " 候选缺少赛事范围 ").unwrap_err(), PersistenceError::InvalidState(message)
        if message == " 候选缺少赛事范围 ")
    );
}

#[test]
fn ledger_and_readiness_counts_reject_negative_without_clamping() {
    assert_eq!(non_negative_u64(0, "样本").unwrap(), 0);
    assert_eq!(non_negative_u64(i64::MAX, "样本").unwrap(), i64::MAX as u64);
    assert!(
        matches!(non_negative_u64(-1, "影子验证样本数").unwrap_err(), PersistenceError::InvalidState(message)
        if message == "影子验证样本数不能为负数")
    );
}
