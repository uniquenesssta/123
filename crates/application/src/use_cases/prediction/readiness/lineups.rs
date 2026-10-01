use super::report::readiness_check;
use football_domain::{MatchLineupChain, PredictionReadinessCheck, PredictionReadinessCheckStatus};
use serde_json::{json, Value};

pub(super) fn selected_lineup(
    chain: &football_domain::MatchLineupTeamChain,
) -> Option<&football_domain::LineupRecord> {
    let selected_id = chain.selected_lineup_id?;
    chain
        .versions
        .iter()
        .find(|lineup| lineup.id == selected_id)
}

pub(super) fn append_unavailable_lineup_readiness_checks(
    checks: &mut Vec<PredictionReadinessCheck>,
    reason: &str,
) {
    for (code, label) in [("home_lineup", "主队阵容"), ("away_lineup", "客队阵容")] {
        checks.push(readiness_check(
            (code, label),
            PredictionReadinessCheckStatus::Blocked,
            15,
            0,
            "赛前数据窗口不可用，尚未选择有效阵容",
            vec![reason.to_string()],
            Value::Null,
        ));
    }
    checks.push(readiness_check(
        ("starting_goalkeepers", "首发门将"),
        PredictionReadinessCheckStatus::Blocked,
        10,
        0,
        "阵容不可用，无法确认双方首发门将",
        vec![reason.to_string()],
        Value::Null,
    ));
    checks.push(readiness_check(
        ("starter_context", "首发位置、角色与状态"),
        PredictionReadinessCheckStatus::Blocked,
        10,
        0,
        "阵容不可用，无法核对首发位置、角色与状态",
        vec![reason.to_string()],
        Value::Null,
    ));
}

pub(super) fn append_lineup_readiness_checks(
    checks: &mut Vec<PredictionReadinessCheck>,
    chain: &MatchLineupChain,
) {
    for (code, label, side) in [
        ("home_lineup", "主队阵容", &chain.home),
        ("away_lineup", "客队阵容", &chain.away),
    ] {
        let Some(lineup) = selected_lineup(side) else {
            checks.push(readiness_check(
                (code, label),
                PredictionReadinessCheckStatus::Blocked,
                15,
                0,
                "当前时间窗口没有可进入模型的阵容",
                side.blocking_issues.clone(),
                json!({"team_id": side.team_id, "team_name": side.team_name}),
            ));
            continue;
        };
        let mut details = lineup.validation_warnings.clone();
        let status = if lineup.lineup_type.as_str() == "confirmed" && details.is_empty() {
            PredictionReadinessCheckStatus::Passed
        } else {
            if lineup.lineup_type.as_str() == "expected" {
                details.push("当前使用预计阵容，正式首发尚未确认".to_string());
            }
            PredictionReadinessCheckStatus::Warning
        };
        checks.push(readiness_check(
            (code, label),
            status,
            15,
            if status == PredictionReadinessCheckStatus::Passed {
                15
            } else {
                12
            },
            if status == PredictionReadinessCheckStatus::Passed {
                "确认阵容完整且通过模型资格校验"
            } else {
                "阵容可用，但仍有需要人工关注的信息"
            },
            details,
            json!({
                "team_id": side.team_id,
                "team_name": side.team_name,
                "lineup_id": lineup.id,
                "lineup_type": lineup.lineup_type.as_str(),
                "captured_at": lineup.captured_at,
                "formation_id": lineup.formation_id,
                "coach_id": lineup.coach_id,
                "player_count": lineup.player_count,
                "starter_count": lineup.starter_count,
                "quality_score": lineup.quality_score,
            }),
        ));
    }

    let selected = [selected_lineup(&chain.home), selected_lineup(&chain.away)];
    let mut goalkeeper_details = Vec::new();
    let mut missing_position_details = Vec::new();
    let mut missing_role_count = 0_usize;
    let mut inherited_role_count = 0_usize;
    let mut overridden_role_count = 0_usize;
    let mut missing_availability_count = 0_usize;
    let mut uncertain_availability_count = 0_usize;
    let mut unavailable_starter_details = Vec::new();
    for (team_name, lineup) in [
        (chain.home.team_name.as_str(), selected[0]),
        (chain.away.team_name.as_str(), selected[1]),
    ] {
        let Some(lineup) = lineup else {
            goalkeeper_details.push(format!("{team_name}尚未选定有效阵容"));
            continue;
        };
        let starters = lineup
            .players
            .iter()
            .filter(|player| player.is_starter)
            .collect::<Vec<_>>();
        let goalkeeper_count = starters
            .iter()
            .filter(|player| {
                player
                    .position_code
                    .as_deref()
                    .is_some_and(|code| code.eq_ignore_ascii_case("GK"))
            })
            .count();
        if goalkeeper_count != 1 {
            goalkeeper_details.push(format!(
                "{team_name}首发必须且只能包含 1 名门将，当前识别为 {goalkeeper_count} 名"
            ));
        }
        for player in starters {
            if player
                .position_code
                .as_deref()
                .is_none_or(|value| value.trim().is_empty())
            {
                missing_position_details.push(format!(
                    "{team_name}首发 {} 未填写实际位置",
                    player.player_name
                ));
            }
            if player
                .role_code
                .as_deref()
                .is_none_or(|value| value.trim().is_empty())
            {
                missing_role_count += 1;
            } else if player.role_origin == "player_position_default" {
                inherited_role_count += 1;
            } else if player.role_origin == "lineup_override" {
                overridden_role_count += 1;
            }
            match player.availability_status {
                None | Some(football_domain::AvailabilityStatus::Unknown) => {
                    missing_availability_count += 1;
                }
                Some(football_domain::AvailabilityStatus::Doubtful)
                | Some(football_domain::AvailabilityStatus::Returning) => {
                    uncertain_availability_count += 1;
                }
                Some(football_domain::AvailabilityStatus::Unavailable)
                | Some(football_domain::AvailabilityStatus::Injured)
                | Some(football_domain::AvailabilityStatus::Suspended)
                | Some(football_domain::AvailabilityStatus::Rested) => {
                    unavailable_starter_details.push(format!(
                        "{team_name}首发 {} 的本场状态为 {}",
                        player.player_name,
                        player
                            .availability_status
                            .map(football_domain::AvailabilityStatus::as_str)
                            .unwrap_or("unknown")
                    ));
                }
                Some(football_domain::AvailabilityStatus::Available) => {}
            }
        }
    }
    checks.push(readiness_check(
        ("starting_goalkeepers", "首发门将"),
        if goalkeeper_details.is_empty() {
            PredictionReadinessCheckStatus::Passed
        } else {
            PredictionReadinessCheckStatus::Blocked
        },
        10,
        if goalkeeper_details.is_empty() { 10 } else { 0 },
        if goalkeeper_details.is_empty() {
            "双方首发门将身份明确"
        } else {
            "首发门将身份不完整"
        },
        goalkeeper_details,
        Value::Null,
    ));

    let player_detail_status =
        if !missing_position_details.is_empty() || !unavailable_starter_details.is_empty() {
            PredictionReadinessCheckStatus::Blocked
        } else if missing_role_count > 0
            || missing_availability_count > 0
            || uncertain_availability_count > 0
        {
            PredictionReadinessCheckStatus::Warning
        } else {
            PredictionReadinessCheckStatus::Passed
        };
    let mut player_details = missing_position_details;
    player_details.extend(unavailable_starter_details);
    if missing_role_count > 0 {
        player_details.push(format!(
            "{missing_role_count} 名首发既没有本场角色覆盖，也没有可继承的球员位置默认角色"
        ));
    }
    if inherited_role_count > 0 {
        player_details.push(format!(
            "{inherited_role_count} 名首发已从球员位置档案自动继承默认战术角色"
        ));
    }
    if overridden_role_count > 0 {
        player_details.push(format!(
            "{overridden_role_count} 名首发使用本场或阵容预设角色覆盖"
        ));
    }
    if missing_availability_count > 0 {
        player_details.push(format!(
            "{missing_availability_count} 名首发缺少明确的本场可用状态快照"
        ));
    }
    if uncertain_availability_count > 0 {
        player_details.push(format!(
            "{uncertain_availability_count} 名首发处于存疑或恢复中状态"
        ));
    }
    checks.push(readiness_check(
        ("starter_context", "首发位置、角色与状态"),
        player_detail_status,
        10,
        match player_detail_status {
            PredictionReadinessCheckStatus::Passed => 10,
            PredictionReadinessCheckStatus::Warning => 7,
            PredictionReadinessCheckStatus::Blocked => 0,
        },
        match player_detail_status {
            PredictionReadinessCheckStatus::Passed => "双方首发位置、角色与可用状态完整",
            PredictionReadinessCheckStatus::Warning => "首发位置完整，但角色或可用状态仍有缺口",
            PredictionReadinessCheckStatus::Blocked => {
                "首发实际位置缺失或存在不可用球员，无法建立可靠的阵型与角色输入"
            }
        },
        player_details,
        json!({
            "missing_role_count": missing_role_count,
            "inherited_role_count": inherited_role_count,
            "overridden_role_count": overridden_role_count,
            "missing_availability_count": missing_availability_count,
            "uncertain_availability_count": uncertain_availability_count,
        }),
    ));
}
