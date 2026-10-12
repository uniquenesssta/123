use crate::{PersistenceError, PersistenceResult};

pub(super) fn require_pending_candidate(status: &str) -> PersistenceResult<()> {
    if status != "pending" {
        return Err(PersistenceError::InvalidState(format!(
            "能力更新候选已经处理，当前状态：{status}"
        )));
    }
    Ok(())
}

pub(super) fn decision_actor(actor: Option<&str>) -> &str {
    actor
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("local_user")
}

pub(super) fn require_current_projection(
    current_value: Option<f64>,
    current_projection: Option<f64>,
) -> PersistenceResult<()> {
    let candidate_is_current = match (current_value, current_projection) {
        (Some(expected), Some(actual)) => (expected - actual).abs() <= 0.01,
        (None, None) => true,
        _ => false,
    };
    if !candidate_is_current {
        return Err(PersistenceError::InvalidState(
            "球员当前能力已发生变化，该候选已经过期，请重新生成复盘".to_string(),
        ));
    }
    Ok(())
}
