use crate::{PersistenceError, PersistenceResult};
use football_domain::PlayerDraft;

pub(super) struct ValidatedPlayerDraft<'a> {
    pub canonical_name: &'a str,
    pub nationality_code: Option<&'a str>,
}

pub(super) fn validate_player_draft(
    draft: &PlayerDraft,
) -> PersistenceResult<ValidatedPlayerDraft<'_>> {
    let canonical_name = draft.canonical_name.trim();
    if canonical_name.is_empty() {
        return Err(PersistenceError::InvalidState(
            "球员姓名不能为空".to_string(),
        ));
    }
    if let Some(height) = draft.height_cm {
        if !(120..=230).contains(&height) {
            return Err(PersistenceError::InvalidState(
                "球员身高必须位于 120–230 cm".to_string(),
            ));
        }
    }
    Ok(ValidatedPlayerDraft {
        canonical_name,
        nationality_code: draft
            .nationality_code
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty()),
    })
}
