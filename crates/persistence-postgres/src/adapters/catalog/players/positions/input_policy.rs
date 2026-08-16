use crate::{PersistenceError, PersistenceResult};
use football_domain::PlayerPositionDraft;

pub(super) struct ValidatedPlayerPosition<'a> {
    pub(super) position_code: String,
    pub(super) default_role_code: Option<&'a str>,
}

pub(super) fn validate_player_position_draft(
    draft: &PlayerPositionDraft,
) -> PersistenceResult<ValidatedPlayerPosition<'_>> {
    if !(0.0..=1.0).contains(&draft.proficiency) {
        return Err(PersistenceError::InvalidState(
            "位置熟练度必须位于 0–1".to_string(),
        ));
    }
    if matches!((&draft.valid_from, &draft.valid_to), (Some(start), Some(end)) if end < start) {
        return Err(PersistenceError::InvalidState(
            "球员位置结束日期不能早于开始日期".to_string(),
        ));
    }
    let default_role_code = draft
        .default_role_code
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty());
    if default_role_code.is_some_and(|value| value.chars().count() > 80) {
        return Err(PersistenceError::InvalidState(
            "默认战术角色不能超过 80 个字符".to_string(),
        ));
    }
    Ok(ValidatedPlayerPosition {
        position_code: draft.position_code.trim().to_uppercase(),
        default_role_code,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;
    use uuid::Uuid;

    fn draft() -> PlayerPositionDraft {
        PlayerPositionDraft {
            player_id: Uuid::nil(),
            position_code: " st ".to_string(),
            proficiency: 0.8,
            default_role_code: Some(" advanced-forward ".to_string()),
            is_primary: false,
            valid_from: None,
            valid_to: None,
            source_document_id: None,
        }
    }

    #[test]
    fn normalizes_position_code_and_role() {
        let value = draft();
        let validated = validate_player_position_draft(&value).unwrap();
        assert_eq!(validated.position_code, "ST");
        assert_eq!(validated.default_role_code, Some("advanced-forward"));
    }

    #[test]
    fn preserves_existing_validation_errors() {
        let mut value = draft();
        value.proficiency = 1.1;
        assert!(matches!(
        validate_player_position_draft(&value),
        Err(PersistenceError::InvalidState(message)) if message == "位置熟练度必须位于 0–1"
              ));
        let mut value = draft();
        value.valid_from = NaiveDate::from_ymd_opt(2026, 2, 2);
        value.valid_to = NaiveDate::from_ymd_opt(2026, 2, 1);
        assert!(matches!(
        validate_player_position_draft(&value),
        Err(PersistenceError::InvalidState(message)) if message == "球员位置结束日期不能早于开始日期"
              ));
        let mut value = draft();
        value.default_role_code = Some("x".repeat(81));
        assert!(matches!(
        validate_player_position_draft(&value),
        Err(PersistenceError::InvalidState(message)) if message == "默认战术角色不能超过 80 个字符"
              ));
    }
}
