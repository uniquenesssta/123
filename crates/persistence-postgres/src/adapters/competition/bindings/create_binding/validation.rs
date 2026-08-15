use football_domain::CompetitionBindingDraft;

use crate::{PersistenceError, PersistenceResult};

pub(super) fn validate_binding_draft(draft: &CompetitionBindingDraft) -> PersistenceResult<()> {
    if draft.competition_id.is_none()
        && draft.season_id.is_none()
        && draft.stage_id.is_none()
        && draft.competition_kind.is_none()
    {
        return Err(PersistenceError::InvalidState(
            "绑定范围不能为空；至少指定赛事、赛季、阶段或赛事类型".to_string(),
        ));
    }

    if let (Some(from), Some(to)) = (&draft.valid_from, &draft.valid_to) {
        if to < from {
            return Err(PersistenceError::InvalidState(
                "绑定结束时间不能早于开始时间".to_string(),
            ));
        }
    }

    Ok(())
}
