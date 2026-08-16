use crate::{PersistenceError, PersistenceResult};
use football_domain::PlayerNameDraft;

pub(super) struct ValidatedPlayerName<'a> {
    pub(super) name: &'a str,
    pub(super) language_code: Option<&'a str>,
}

pub(super) fn validate_player_name_draft(
    draft: &PlayerNameDraft,
) -> PersistenceResult<ValidatedPlayerName<'_>> {
    let name = draft.name.trim();
    if name.is_empty() {
        return Err(PersistenceError::InvalidState(
            "球员名称不能为空".to_string(),
        ));
    }
    if matches!((&draft.valid_from, &draft.valid_to), (Some(start), Some(end)) if end < start) {
        return Err(PersistenceError::InvalidState(
            "球员名称结束日期不能早于开始日期".to_string(),
        ));
    }
    Ok(ValidatedPlayerName {
        name,
        language_code: draft
            .language_code
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty()),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;
    use uuid::Uuid;

    fn draft(name: &str) -> PlayerNameDraft {
        PlayerNameDraft {
            player_id: Uuid::nil(),
            name: name.to_string(),
            language_code: Some(" zh-CN ".to_string()),
            is_primary: false,
            valid_from: None,
            valid_to: None,
        }
    }

    #[test]
    fn trims_name_and_optional_language() {
        let value = draft("  测试 球员  ");
        let validated = validate_player_name_draft(&value).unwrap();
        assert_eq!(validated.name, "测试 球员");
        assert_eq!(validated.language_code, Some("zh-CN"));
    }

    #[test]
    fn rejects_blank_name_and_reversed_dates() {
        assert!(matches!(
        validate_player_name_draft(&draft("   ")),
        Err(PersistenceError::InvalidState(message)) if message == "球员名称不能为空"
              ));
        let mut value = draft("name");
        value.valid_from = NaiveDate::from_ymd_opt(2026, 2, 2);
        value.valid_to = NaiveDate::from_ymd_opt(2026, 2, 1);
        assert!(matches!(
        validate_player_name_draft(&value),
        Err(PersistenceError::InvalidState(message)) if message == "球员名称结束日期不能早于开始日期"
              ));
    }
}
