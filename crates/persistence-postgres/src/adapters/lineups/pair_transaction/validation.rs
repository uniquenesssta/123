use crate::{PersistenceError, PersistenceResult};
use football_domain::LineupDraft;
use std::collections::HashSet;
use uuid::Uuid;

pub(super) struct ValidatedLineupDraft {
    pub(super) snapshot_type: String,
    pub(super) starters: usize,
}

pub(super) fn validate_lineup_draft(
    draft: &LineupDraft,
) -> PersistenceResult<ValidatedLineupDraft> {
    if !(1..=30).contains(&draft.players.len()) {
        return Err(PersistenceError::InvalidState(
            "阵容球员数量必须位于 1–30".to_string(),
        ));
    }
    let unique_players: HashSet<Uuid> = draft
        .players
        .iter()
        .map(|player| player.player_id)
        .collect();
    if unique_players.len() != draft.players.len() {
        return Err(PersistenceError::InvalidState(
            "同一阵容中存在重复球员".to_string(),
        ));
    }
    let starters = draft
        .players
        .iter()
        .filter(|player| player.is_starter)
        .count();
    if starters != 11 {
        return Err(PersistenceError::InvalidState(format!(
            "正式阵容必须恰好 11 名首发，当前为 {starters} 名",
        )));
    }
    if draft
        .quality_score
        .is_some_and(|value| !(0.0..=1.0).contains(&value))
    {
        return Err(PersistenceError::InvalidState(
            "阵容质量分必须位于 0–1".to_string(),
        ));
    }
    let snapshot_type =
        crate::adapters::lineups::chain::normalize_lineup_snapshot_type(&draft.snapshot_type)?
            .to_string();
    for player in &draft.players {
        if player
            .shirt_number
            .is_some_and(|number| !(0..=99).contains(&number))
        {
            return Err(PersistenceError::InvalidState(
                "阵容球衣号码必须位于 0–99".to_string(),
            ));
        }
        if player
            .expected_minutes
            .is_some_and(|minutes| !(0..=150).contains(&minutes))
            || player
                .actual_minutes
                .is_some_and(|minutes| !(0..=150).contains(&minutes))
        {
            return Err(PersistenceError::InvalidState(
                "阵容分钟数必须位于 0–150".to_string(),
            ));
        }
        if player.sequence_no < 0 {
            return Err(PersistenceError::InvalidState(
                "阵容排序号不能为负数".to_string(),
            ));
        }
        if player
            .bench_order
            .is_some_and(|value| !(1..=99).contains(&value))
        {
            return Err(PersistenceError::InvalidState(
                "替补顺序必须位于 1–99".to_string(),
            ));
        }
        if player
            .starting_probability
            .is_some_and(|value| !(0.0..=1.0).contains(&value))
        {
            return Err(PersistenceError::InvalidState(
                "首发概率必须位于 0–1".to_string(),
            ));
        }
    }
    Ok(ValidatedLineupDraft {
        snapshot_type,
        starters,
    })
}
#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use football_domain::{LineupPlayerDraft, LineupType};
    use serde_json::json;

    fn draft() -> LineupDraft {
        LineupDraft {
            match_id: Uuid::new_v4(),
            team_id: Uuid::new_v4(),
            lineup_type: LineupType::Expected,
            snapshot_type: " T-6h ".into(),
            formation: Some("4-2-3-1".into()),
            formation_id: Some(Uuid::new_v4()),
            coach_id: None,
            captured_at: Utc::now(),
            source_document_id: None,
            source_urls: vec![],
            quality_score: Some(0.9),
            metadata: json!({}),
            players: (0..13)
                .map(|index| LineupPlayerDraft {
                    player_id: Uuid::new_v4(),
                    position_code: Some(if index == 0 { "GK" } else { "CM" }.into()),
                    role_code: None,
                    is_starter: index < 11,
                    shirt_number: Some(index + 1),
                    expected_minutes: Some(90),
                    actual_minutes: None,
                    sequence_no: index + 1,
                    bench_order: if index >= 11 { Some(index - 10) } else { None },
                    availability_status: None,
                    starting_probability: Some(if index < 11 { 1.0 } else { 0.0 }),
                    membership_override: false,
                    source_urls: vec![],
                    metadata: json!({}),
                })
                .collect(),
        }
    }

    #[test]
    fn legal_starters_and_bench_keep_the_normalized_window() {
        let validated = validate_lineup_draft(&draft()).unwrap();
        assert_eq!(validated.starters, 11);
        assert_eq!(validated.snapshot_type, "T-6h");
    }

    #[test]
    fn duplicate_identity_and_missing_starters_are_rejected() {
        let mut duplicate = draft();
        duplicate.players[12].player_id = duplicate.players[0].player_id;
        assert!(
            matches!(validate_lineup_draft(&duplicate), Err(PersistenceError::InvalidState(message)) if message.contains("重复球员"))
        );
        let mut incomplete = draft();
        incomplete.players[10].is_starter = false;
        assert!(
            matches!(validate_lineup_draft(&incomplete), Err(PersistenceError::InvalidState(message)) if message.contains("11 名首发"))
        );
    }

    #[test]
    fn legacy_window_and_invalid_numeric_inputs_remain_rejected() {
        let mut legacy = draft();
        legacy.snapshot_type = "T-90m".into();
        assert!(validate_lineup_draft(&legacy).is_err());
        let mut invalid = draft();
        invalid.quality_score = Some(f64::NAN);
        assert!(validate_lineup_draft(&invalid).is_err());
        let mut invalid = draft();
        invalid.players[11].bench_order = Some(100);
        assert!(validate_lineup_draft(&invalid).is_err());
        let mut invalid = draft();
        invalid.players[0].starting_probability = Some(f64::NAN);
        assert!(validate_lineup_draft(&invalid).is_err());
    }
}
