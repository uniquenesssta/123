use crate::{PersistenceError, PersistenceResult};
use football_domain::DataProviderDraft;

pub(super) struct ValidatedProvider<'a> {
    pub code: String,
    pub name: &'a str,
    pub provider_type: &'a str,
    pub base_url: Option<&'a str>,
}

pub(super) fn validate_provider(
    draft: &DataProviderDraft,
) -> PersistenceResult<ValidatedProvider<'_>> {
    let code = draft.code.trim().to_lowercase();
    let name = draft.name.trim();
    let provider_type = draft.provider_type.trim();
    if code.is_empty() || name.is_empty() || provider_type.is_empty() {
        return Err(PersistenceError::InvalidState(
            "数据源代码、名称和类型不能为空".to_string(),
        ));
    }
    Ok(ValidatedProvider {
        code,
        name,
        provider_type,
        base_url: draft
            .base_url
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty()),
    })
}
