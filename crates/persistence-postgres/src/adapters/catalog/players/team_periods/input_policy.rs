use crate::{PersistenceError, PersistenceResult};
use football_domain::PlayerTeamPeriodDraft;

pub(super) struct ValidatedPlayerTeamPeriodInput<'a> {
    pub registration_status: &'a str,
}

pub(super) fn validate_player_team_period_draft(
    draft: &PlayerTeamPeriodDraft,
) -> PersistenceResult<ValidatedPlayerTeamPeriodInput<'_>> {
    if draft
        .valid_to
        .as_ref()
        .is_some_and(|valid_to| valid_to < &draft.valid_from)
    {
        return Err(PersistenceError::InvalidState(
            "球队效力结束日期不能早于开始日期".to_string(),
        ));
    }
    if draft
        .squad_number
        .is_some_and(|number| !(0..=99).contains(&number))
    {
        return Err(PersistenceError::InvalidState(
            "球衣号码必须位于 0–99".to_string(),
        ));
    }
    Ok(ValidatedPlayerTeamPeriodInput {
        registration_status: draft.registration_status.trim(),
    })
}
