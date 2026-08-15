use crate::{PersistenceError, PersistenceResult};
use football_domain::TeamNameDraft;

pub(super) fn validated_name(draft: &TeamNameDraft) -> PersistenceResult<&str> {
    let name = draft.name.trim();
    if name.is_empty() {
        return Err(PersistenceError::InvalidState(
            "球队别名不能为空".to_string(),
        ));
    }
    if let (Some(from), Some(to)) = (draft.valid_from, draft.valid_to) {
        if to < from {
            return Err(PersistenceError::InvalidState(
                "球队别名结束日期早于开始日期".to_string(),
            ));
        }
    }
    Ok(name)
}

pub(super) fn normalized_language_code(draft: &TeamNameDraft) -> Option<&str> {
    draft
        .language_code
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
}

#[cfg(test)]
mod tests {
    use super::{normalized_language_code, validated_name};
    use crate::PersistenceError;
    use chrono::NaiveDate;
    use football_domain::TeamNameDraft;
    use uuid::Uuid;

    fn draft(name: &str) -> TeamNameDraft {
        TeamNameDraft {
            team_id: Uuid::nil(),
            name: name.to_string(),
            language_code: None,
            valid_from: None,
            valid_to: None,
        }
    }

    #[test]
    fn rejects_blank_name_with_existing_error_semantics() {
        let error = validated_name(&draft("   ")).expect_err("blank alias must fail");
        assert!(matches!(
            error,
            PersistenceError::InvalidState(message) if message == "球队别名不能为空"
        ));
    }

    #[test]
    fn rejects_reversed_validity_range() {
        let mut value = draft("Alias");
        value.valid_from = Some(NaiveDate::from_ymd_opt(2026, 8, 2).unwrap());
        value.valid_to = Some(NaiveDate::from_ymd_opt(2026, 8, 1).unwrap());
        let error = validated_name(&value).expect_err("reversed range must fail");
        assert!(matches!(
            error,
            PersistenceError::InvalidState(message) if message == "球队别名结束日期早于开始日期"
        ));
    }

    #[test]
    fn trims_name_and_language_code_without_changing_draft_contract() {
        let mut value = draft("  Alias  ");
        value.language_code = Some("  zh-CN  ".to_string());
        assert_eq!(validated_name(&value).unwrap(), "Alias");
        assert_eq!(normalized_language_code(&value), Some("zh-CN"));
    }
}
