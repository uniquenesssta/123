use crate::{PersistenceError, PersistenceResult};
use football_domain::PlayerAbilityObservationDraft;

pub(super) fn validate_player_ability_observation(
    draft: &PlayerAbilityObservationDraft,
) -> PersistenceResult<()> {
    if !(0.0..=1.0).contains(&draft.confidence) || draft.sample_size < 0 {
        return Err(PersistenceError::InvalidState(
            "能力观察可信度或样本量无效".to_string(),
        ));
    }
    if draft
        .effective_to
        .as_ref()
        .is_some_and(|value| value < &draft.effective_from)
    {
        return Err(PersistenceError::InvalidState(
            "能力观察失效时间不能早于生效时间".to_string(),
        ));
    }
    Ok(())
}
