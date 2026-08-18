use crate::{PersistenceError, PersistenceResult};
use football_domain::{CoachDraft, TeamCoachPeriodDraft};
pub(crate) fn validate_coach_draft(draft: &CoachDraft) -> PersistenceResult<()> {
    if draft.canonical_name.trim().is_empty() {
        return Err(PersistenceError::InvalidState(
            "教练姓名不能为空".to_string(),
        ));
    }
    if !matches!(
        draft.status.trim(),
        "active" | "inactive" | "retired" | "unknown"
    ) {
        return Err(PersistenceError::InvalidState("教练状态无效".to_string()));
    }
    Ok(())
}

pub(crate) fn validate_team_coach_period(draft: &TeamCoachPeriodDraft) -> PersistenceResult<()> {
    if !matches!(
        draft.role.trim(),
        "head_coach" | "assistant_coach" | "interim_head_coach" | "caretaker" | "other"
    ) {
        return Err(PersistenceError::InvalidState("教练职务无效".to_string()));
    }
    validate_date_range(Some(draft.valid_from), draft.valid_to, "教练任期")?;
    if !(0.0..=1.0).contains(&draft.confidence) {
        return Err(PersistenceError::InvalidState(
            "教练任期可信度必须在0到1之间".to_string(),
        ));
    }
    Ok(())
}

pub(crate) fn validate_date_range(
    from: Option<chrono::NaiveDate>,
    to: Option<chrono::NaiveDate>,
    label: &str,
) -> PersistenceResult<()> {
    if let (Some(from), Some(to)) = (from, to) {
        if to < from {
            return Err(PersistenceError::InvalidState(format!(
                "{label}结束日期早于开始日期"
            )));
        }
    }
    Ok(())
}
