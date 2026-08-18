use crate::{PersistenceError, PersistenceResult};
use football_domain::ExternalEntityIdDraft;

pub(super) struct ValidatedExternalId<'a> {
    pub entity_type: &'a str,
    pub external_id: &'a str,
}

pub(super) fn validate_external_id(
    draft: &ExternalEntityIdDraft,
) -> PersistenceResult<ValidatedExternalId<'_>> {
    if !matches!(
        draft.entity_type.as_str(),
        "competition" | "season" | "team" | "player" | "coach" | "match"
    ) {
        return Err(PersistenceError::InvalidState(
            "外部 ID 实体类型无效".to_string(),
        ));
    }
    let external_id = draft.external_id.trim();
    if external_id.is_empty() {
        return Err(PersistenceError::InvalidState(
            "外部 ID 不能为空".to_string(),
        ));
    }
    Ok(ValidatedExternalId {
        entity_type: draft.entity_type.trim(),
        external_id,
    })
}
