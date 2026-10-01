use crate::{PersistenceError, PersistenceResult};
use football_domain::TeamLineupPresetDraft;
use sqlx::{Postgres, Transaction};
use std::collections::HashSet;
use uuid::Uuid;
pub(super) fn validate_preset(draft: &TeamLineupPresetDraft) -> PersistenceResult<()> {
    if draft.name.trim().is_empty() {
        return Err(PersistenceError::InvalidState(
            "阵容预设名称不能为空".to_string(),
        ));
    }
    if let Some(probability) = draft.usage_probability {
        if !(0.0..=1.0).contains(&probability) {
            return Err(PersistenceError::InvalidState(
                "阵容预设使用概率必须在 0 到 1 之间".to_string(),
            ));
        }
    }
    if draft.members.len() < 11 {
        return Err(PersistenceError::InvalidState(
            "阵容预设至少需要 11 名球员".to_string(),
        ));
    }
    let starter_count = draft
        .members
        .iter()
        .filter(|member| member.is_starter)
        .count();
    if starter_count != 11 {
        return Err(PersistenceError::InvalidState(format!(
            "阵容预设必须恰好包含 11 名首发，当前为 {starter_count} 名"
        )));
    }
    let unique_players = draft
        .members
        .iter()
        .map(|member| member.player_id)
        .collect::<HashSet<_>>();
    if unique_players.len() != draft.members.len() {
        return Err(PersistenceError::InvalidState(
            "阵容预设不能包含重复球员".to_string(),
        ));
    }
    if draft
        .members
        .iter()
        .filter(|member| member.is_captain)
        .count()
        > 1
    {
        return Err(PersistenceError::InvalidState(
            "阵容预设最多只能设置一名队长".to_string(),
        ));
    }
    Ok(())
}

pub(super) async fn verify_membership_in_tx(
    tx: &mut Transaction<'_, Postgres>,
    team_id: Uuid,
    player_ids: &[Uuid],
) -> PersistenceResult<()> {
    for player_id in player_ids {
        let belongs: bool = sqlx::query_scalar(
            r#"
            SELECT EXISTS (
                SELECT 1
                FROM football.player_team_periods period
                WHERE period.player_id = $1
                  AND period.team_id = $2
                  AND period.valid_from <= current_date
                  AND (period.valid_to IS NULL OR period.valid_to >= current_date)
                  AND period.registration_status IN ('registered', 'loan', 'trial')
            )
            "#,
        )
        .bind(player_id)
        .bind(team_id)
        .fetch_one(&mut **tx)
        .await?;
        if !belongs {
            return Err(PersistenceError::InvalidState(format!(
                "球员 {player_id} 当前不属于该球队，不能保存到活动阵容预设"
            )));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use football_domain::TeamLineupPresetMemberDraft;
    use serde_json::json;

    fn member(index: u128, starter: bool) -> TeamLineupPresetMemberDraft {
        TeamLineupPresetMemberDraft {
            player_id: Uuid::from_u128(index + 1),
            position_code: None,
            role_code: None,
            is_starter: starter,
            shirt_number: None,
            expected_minutes: Some(if starter { 90 } else { 20 }),
            sequence_no: index as i16,
            bench_order: if starter { None } else { Some(index as i16) },
            is_captain: index == 0,
            metadata: json!({}),
        }
    }

    fn draft(starter_count: usize, member_count: usize) -> TeamLineupPresetDraft {
        TeamLineupPresetDraft {
            id: None,
            team_id: Uuid::from_u128(10_000),
            name: "主力阵容".to_string(),
            formation_id: None,
            coach_id: None,
            usage_context: "general".to_string(),
            usage_probability: Some(0.6),
            is_default: true,
            source_lineup_id: None,
            notes: None,
            members: (0..member_count)
                .map(|index| member(index as u128, index < starter_count))
                .collect(),
        }
    }

    #[test]
    fn preset_requires_exactly_eleven_starters() {
        assert!(validate_preset(&draft(11, 18)).is_ok());
        assert!(validate_preset(&draft(10, 18)).is_err());
        assert!(validate_preset(&draft(12, 18)).is_err());
    }

    #[test]
    fn preset_rejects_duplicate_players() {
        let mut value = draft(11, 18);
        value.members[17].player_id = value.members[0].player_id;
        assert!(validate_preset(&value).is_err());
    }

    #[test]
    fn preset_validates_name_probability_and_minimum_members() {
        let mut value = draft(11, 11);
        for probability in [None, Some(0.0), Some(1.0)] {
            value.usage_probability = probability;
            assert!(validate_preset(&value).is_ok());
        }
        for probability in [-0.01, 1.01, f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            value.usage_probability = Some(probability);
            assert!(validate_preset(&value).is_err());
        }
        value.usage_probability = None;
        value.name = " \t\n ".into();
        assert!(validate_preset(&value).is_err());
        assert!(validate_preset(&draft(10, 10)).is_err());
    }

    #[test]
    fn preset_accepts_no_captain_but_rejects_multiple_captains() {
        let mut value = draft(11, 18);
        value.members[0].is_captain = false;
        assert!(validate_preset(&value).is_ok());
        value.members[0].is_captain = true;
        value.members[17].is_captain = true;
        assert!(validate_preset(&value).is_err());
    }
}
