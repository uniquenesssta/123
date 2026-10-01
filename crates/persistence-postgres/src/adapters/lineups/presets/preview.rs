use crate::{PersistenceResult, PostgresStore};
use football_domain::{
    AvailabilityStatus, TeamLineupPresetApplicationPreview, TeamLineupPresetRecord,
};
use uuid::Uuid;

impl PostgresStore {
    pub async fn preview_team_lineup_preset_application(
        &self,
        preset_id: Uuid,
    ) -> PersistenceResult<TeamLineupPresetApplicationPreview> {
        let preset = self.read_team_lineup_preset(preset_id).await?;
        Ok(assess_application(preset))
    }
}

// 只评估已读取的预设；没有写入、审计或正式阵容提交能力。
fn assess_application(preset: TeamLineupPresetRecord) -> TeamLineupPresetApplicationPreview {
    let mut blockers = Vec::new();
    let mut warnings = Vec::new();
    if preset.status != "active" {
        blockers.push("阵容预设已经归档".to_string());
    }
    if preset.starter_count != 11 {
        blockers.push(format!(
            "预设必须恰好包含 11 名首发，当前为 {} 名",
            preset.starter_count
        ));
    }
    for member in &preset.members {
        if member.player_status != "active" {
            blockers.push(format!("{} 已不是活动球员", member.player_name));
        }
        if member.current_team_id != Some(preset.team_id) {
            blockers.push(format!(
                "{} 当前不再属于 {}",
                member.player_name, preset.team_name
            ));
        }
        if matches!(
            member.availability_status,
            Some(AvailabilityStatus::Injured)
                | Some(AvailabilityStatus::Suspended)
                | Some(AvailabilityStatus::Unavailable)
                | Some(AvailabilityStatus::Doubtful)
        ) {
            warnings.push(format!(
                "{} 当前状态为 {}",
                member.player_name,
                member
                    .availability_status
                    .map(AvailabilityStatus::as_str)
                    .unwrap_or("unknown")
            ));
        }
    }
    blockers.sort();
    blockers.dedup();
    warnings.sort();
    warnings.dedup();
    TeamLineupPresetApplicationPreview {
        can_apply: blockers.is_empty(),
        preset,
        blockers,
        warnings,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use football_domain::TeamLineupPresetMemberRecord;
    use serde_json::json;

    fn preset() -> TeamLineupPresetRecord {
        let team_id = Uuid::from_u128(100);
        let now = Utc::now();
        TeamLineupPresetRecord {
            id: Uuid::from_u128(200),
            team_id,
            team_name: "主队".into(),
            name: "主力".into(),
            formation_id: None,
            formation_code: None,
            formation_name: None,
            coach_id: None,
            coach_name: None,
            usage_context: "general".into(),
            usage_probability: Some(0.6),
            is_default: true,
            status: "active".into(),
            version: 3,
            source_lineup_id: None,
            notes: Some("保持备注".into()),
            starter_count: 11,
            member_count: 11,
            members: (0..11)
                .map(|index| TeamLineupPresetMemberRecord {
                    player_id: Uuid::from_u128(index + 1),
                    player_name: format!("球员{index}"),
                    alternate_name: None,
                    position_code: Some("CM".into()),
                    role_code: Some("组织核心".into()),
                    role_origin: "player_position_default".into(),
                    role_source_position_code: Some("CM".into()),
                    is_starter: true,
                    shirt_number: None,
                    expected_minutes: Some(90),
                    sequence_no: index as i16,
                    bench_order: None,
                    is_captain: index == 0,
                    current_team_id: Some(team_id),
                    current_team_name: Some("主队".into()),
                    player_status: "active".into(),
                    availability_status: Some(AvailabilityStatus::Available),
                    metadata: json!({"retain": true}),
                })
                .collect(),
            created_at: now,
            updated_at: now,
        }
    }

    #[test]
    fn legal_preview_preserves_entire_preset_and_role_provenance() {
        let source = preset();
        let before = serde_json::to_value(&source).unwrap();
        let preview = assess_application(source);
        assert!(preview.can_apply);
        assert!(preview.blockers.is_empty() && preview.warnings.is_empty());
        assert_eq!(serde_json::to_value(preview.preset).unwrap(), before);
    }

    #[test]
    fn archived_or_wrong_starter_count_blocks_preview() {
        let mut source = preset();
        source.status = "archived".into();
        let archived = assess_application(source.clone());
        assert!(!archived.can_apply);
        assert_eq!(archived.blockers, ["阵容预设已经归档"]);
        source.status = "active".into();
        for count in [10, 12] {
            source.starter_count = count;
            let preview = assess_application(source.clone());
            assert!(!preview.can_apply);
            assert_eq!(preview.blockers.len(), 1);
            assert!(preview.blockers[0].contains(&format!("当前为 {count} 名")));
        }
    }

    #[test]
    fn inactive_and_departed_members_block_preview() {
        let mut source = preset();
        source.members[0].player_status = "inactive".into();
        source.members[1].current_team_id = None;
        source.members[2].current_team_id = Some(Uuid::from_u128(101));
        let preview = assess_application(source);
        assert!(!preview.can_apply);
        assert_eq!(preview.blockers.len(), 3);
        assert!(preview
            .blockers
            .iter()
            .any(|message| message == "球员0 已不是活动球员"));
        assert!(preview
            .blockers
            .iter()
            .any(|message| message == "球员1 当前不再属于 主队"));
        assert!(preview
            .blockers
            .iter()
            .any(|message| message == "球员2 当前不再属于 主队"));
    }

    #[test]
    fn availability_is_a_warning_and_does_not_block_application() {
        for status in [
            AvailabilityStatus::Injured,
            AvailabilityStatus::Suspended,
            AvailabilityStatus::Unavailable,
            AvailabilityStatus::Doubtful,
        ] {
            let mut source = preset();
            source.members[0].availability_status = Some(status);
            let preview = assess_application(source);
            assert!(preview.can_apply && preview.blockers.is_empty());
            assert_eq!(
                preview.warnings,
                [format!("球员0 当前状态为 {}", status.as_str())]
            );
        }
        for status in [
            None,
            Some(AvailabilityStatus::Available),
            Some(AvailabilityStatus::Rested),
            Some(AvailabilityStatus::Returning),
            Some(AvailabilityStatus::Unknown),
        ] {
            let mut source = preset();
            source.members[0].availability_status = status;
            assert!(assess_application(source).warnings.is_empty());
        }
    }

    #[test]
    fn preview_issues_are_sorted_deduplicated_and_repeatable() {
        let mut source = preset();
        for member in &mut source.members {
            member.player_name = "同名球员".into();
            member.player_status = "inactive".into();
            member.current_team_id = None;
            member.availability_status = Some(AvailabilityStatus::Injured);
        }
        let first = assess_application(source.clone());
        let second = assess_application(source);
        assert!(!first.can_apply);
        assert_eq!(first.blockers.len(), 2);
        assert!(first.blockers.windows(2).all(|pair| pair[0] < pair[1]));
        assert_eq!(first.warnings, ["同名球员 当前状态为 injured"]);
        assert_eq!(
            serde_json::to_value(first).unwrap(),
            serde_json::to_value(second).unwrap()
        );
    }
}
