use crate::{PersistenceError, PersistenceResult};
use football_domain::TeamProfileDraft;

pub(super) fn validate_team_profile(draft: &TeamProfileDraft) -> PersistenceResult<()> {
    if draft
        .founded_year
        .is_some_and(|year| !(1850..=2100).contains(&year))
    {
        return Err(PersistenceError::InvalidState(
            "球队成立年份必须在1850到2100之间".to_string(),
        ));
    }
    if !matches!(
        draft.team_type.trim(),
        "club" | "national" | "reserve" | "youth" | "women" | "other"
    ) {
        return Err(PersistenceError::InvalidState("球队类型无效".to_string()));
    }
    if !matches!(
        draft.tactical_style.trim(),
        "balanced" | "possession" | "direct" | "counter" | "pressing" | "defensive" | "custom"
    ) {
        return Err(PersistenceError::InvalidState("战术风格无效".to_string()));
    }
    for (label, value) in [
        ("进攻评分", draft.attack_rating),
        ("中场评分", draft.midfield_rating),
        ("防守评分", draft.defence_rating),
        ("门将评分", draft.goalkeeper_rating),
        ("声望", draft.reputation),
    ] {
        if value.is_some_and(|value| !(0.0..=100.0).contains(&value)) {
            return Err(PersistenceError::InvalidState(format!(
                "{label}必须在0到100之间"
            )));
        }
    }
    if !(0.0..=1.0).contains(&draft.data_confidence) {
        return Err(PersistenceError::InvalidState(
            "球队资料可信度必须在0到1之间".to_string(),
        ));
    }
    Ok(())
}

pub(super) fn trimmed_optional_text(value: &Option<String>) -> Option<&str> {
    value
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
}

#[cfg(test)]
mod tests {
    use super::{trimmed_optional_text, validate_team_profile};
    use crate::PersistenceError;
    use football_domain::TeamProfileDraft;
    use serde_json::json;

    fn valid_draft() -> TeamProfileDraft {
        TeamProfileDraft {
            short_name: Some("  Team  ".to_string()),
            team_type: "club".to_string(),
            founded_year: Some(2000),
            city: None,
            stadium: None,
            head_coach: None,
            default_formation: None,
            tactical_style: "balanced".to_string(),
            attack_rating: Some(80.0),
            midfield_rating: Some(80.0),
            defence_rating: Some(80.0),
            goalkeeper_rating: Some(80.0),
            reputation: Some(80.0),
            data_confidence: 0.9,
            notes: None,
            metadata: json!({}),
        }
    }

    #[test]
    fn preserves_existing_profile_validation_messages() {
        let mut draft = valid_draft();
        draft.team_type = "invalid".to_string();
        let error = validate_team_profile(&draft).expect_err("invalid team type must fail");
        assert!(matches!(
            error,
            PersistenceError::InvalidState(message) if message == "球队类型无效"
        ));

        let mut draft = valid_draft();
        draft.data_confidence = 1.1;
        let error = validate_team_profile(&draft).expect_err("confidence over one must fail");
        assert!(matches!(
            error,
            PersistenceError::InvalidState(message) if message == "球队资料可信度必须在0到1之间"
        ));
    }

    #[test]
    fn trims_optional_text_and_drops_blank_values() {
        assert_eq!(trimmed_optional_text(&Some("  Team  ".to_string())), Some("Team"));
        assert_eq!(trimmed_optional_text(&Some("   ".to_string())), None);
        assert_eq!(trimmed_optional_text(&None), None);
    }
}
