use crate::{PersistenceError, PersistenceResult};
use football_domain::FormationUsageDistributionDraft;
use std::collections::HashSet;

pub(crate) fn validate_distribution_draft(
    draft: &FormationUsageDistributionDraft,
) -> PersistenceResult<()> {
    let valid_shape = match draft.scope_type.trim() {
        "team" => {
            draft.team_id.is_some() && draft.coach_id.is_none() && draft.competition_id.is_none()
        }
        "coach" => {
            draft.team_id.is_none() && draft.coach_id.is_some() && draft.competition_id.is_none()
        }
        "team_coach" => {
            draft.team_id.is_some() && draft.coach_id.is_some() && draft.competition_id.is_none()
        }
        "competition_default" => {
            draft.team_id.is_none() && draft.coach_id.is_none() && draft.competition_id.is_some()
        }
        "system_default" => {
            draft.team_id.is_none() && draft.coach_id.is_none() && draft.competition_id.is_none()
        }
        _ => false,
    };
    if !valid_shape {
        return Err(PersistenceError::InvalidState(
            "阵型概率作用域与球队、教练、赛事字段不匹配".to_string(),
        ));
    }
    if draft.observed_matches < 0 {
        return Err(PersistenceError::InvalidState(
            "观察场数不能为负数".to_string(),
        ));
    }
    if !(0.0..=1.0).contains(&draft.confidence) {
        return Err(PersistenceError::InvalidState(
            "阵型观察可信度必须位于 0–1".to_string(),
        ));
    }
    if !draft.alpha.is_finite() || draft.alpha <= 0.0 || draft.alpha > 100.0 {
        return Err(PersistenceError::InvalidState(
            "阵型平滑参数 alpha 必须位于 0–100".to_string(),
        ));
    }
    let mut ids = HashSet::new();
    if draft
        .entries
        .iter()
        .any(|entry| !ids.insert(entry.formation_id))
    {
        return Err(PersistenceError::InvalidState(
            "阵型概率条目存在重复阵型".to_string(),
        ));
    }
    Ok(())
}
