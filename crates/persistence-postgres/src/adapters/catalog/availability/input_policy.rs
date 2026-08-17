use crate::{PersistenceError, PersistenceResult};
use football_domain::PlayerAvailabilityDraft;

pub(super) struct ValidatedPlayerAvailabilityInput<'a> {
    pub reason: Option<&'a str>,
}

pub(super) fn validate_player_availability_draft(
    draft: &PlayerAvailabilityDraft,
) -> PersistenceResult<ValidatedPlayerAvailabilityInput<'_>> {
    if !(0.0..=1.0).contains(&draft.confidence) {
        return Err(PersistenceError::InvalidState(
            "可用性可信度必须位于 0–1".to_string(),
        ));
    }
    if draft
        .valid_to
        .as_ref()
        .is_some_and(|valid_to| valid_to < &draft.valid_from)
    {
        return Err(PersistenceError::InvalidState(
            "可用性结束时间不能早于开始时间".to_string(),
        ));
    }
    Ok(ValidatedPlayerAvailabilityInput {
        reason: draft
            .reason
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty()),
    })
}
