use crate::{PersistenceError, PersistenceResult};
use football_domain::ParameterTuningDecision;
use uuid::Uuid;

pub(super) fn required_uuid(value: Option<Uuid>, message: &str) -> PersistenceResult<Uuid> {
    value.ok_or_else(|| PersistenceError::InvalidState(message.to_string()))
}

pub(super) fn non_negative_u64(value: i64, label: &str) -> PersistenceResult<u64> {
    u64::try_from(value).map_err(|_| PersistenceError::InvalidState(format!("{label}不能为负数")))
}

pub(super) fn tuning_decision_status(decision: ParameterTuningDecision) -> &'static str {
    match decision {
        ParameterTuningDecision::AcceptForBacktest => "accepted_for_backtest",
        ParameterTuningDecision::Reject => "rejected",
    }
}

pub(super) fn shadow_candidate_status(status: &str) -> &'static str {
    match status {
        "passed" => "shadow_passed",
        "failed" => "shadow_failed",
        _ => "blocked_by_h",
    }
}

pub(super) fn ensure_promotable(status: &str) -> PersistenceResult<()> {
    if status != "shadow_passed" {
        return Err(PersistenceError::InvalidState(
            "只有影子验证通过的候选可以人工晋升".to_string(),
        ));
    }
    Ok(())
}

pub(super) fn ensure_rollback_allowed(status: &str) -> PersistenceResult<()> {
    if status != "promoted" {
        return Err(PersistenceError::InvalidState(
            "只有已晋升且未回滚的候选可以回滚".to_string(),
        ));
    }
    Ok(())
}
