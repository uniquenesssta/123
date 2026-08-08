use football_domain::{MatchLineupChain, PredictionReadinessCheck, PredictionReadinessCheckStatus};
use serde_json::{json, Value};

pub(crate) fn readiness_check(
    (code, label): (&str, &str),
    status: PredictionReadinessCheckStatus,
    weight: u8,
    score: u8,
    summary: &str,
    details: Vec<String>,
    metadata: Value,
) -> PredictionReadinessCheck {
    PredictionReadinessCheck {
        code: code.to_string(),
        label: label.to_string(),
        status,
        weight,
        score: score.min(weight),
        summary: summary.to_string(),
        details,
        metadata,
    }
}

pub(crate) fn selected_lineup(
    chain: &football_domain::MatchLineupTeamChain,
) -> Option<&football_domain::LineupRecord> {
    let selected_id = chain.selected_lineup_id?;
    chain
        .versions
        .iter()
        .find(|lineup| lineup.id == selected_id)
}

pub(crate) fn append_unavailable_lineup_readiness_checks(
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

pub(crate) fn append_lineup_readiness_checks(
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

pub(crate) fn append_unavailable_prepared_input_checks(
    checks: &mut Vec<PredictionReadinessCheck>,
    history_reason: &str,
    input_reason: &str,
) {
    checks.push(readiness_check(
        ("team_history", "球队历史样本"),
        PredictionReadinessCheckStatus::Blocked,
        10,
        0,
        history_reason,
        Vec::new(),
        Value::Null,
    ));
    checks.push(readiness_check(
        ("model_input", "模型输入构建与质量"),
        PredictionReadinessCheckStatus::Blocked,
        5,
        0,
        "数据库事实尚不能构建确定性模型输入",
        vec![input_reason.to_string()],
        Value::Null,
    ));
}

pub(crate) fn append_prepared_input_checks(
    checks: &mut Vec<PredictionReadinessCheck>,
    shadow_reasons: &mut Vec<String>,
    prepared: &football_domain::PreparedMatchPredictionInput,
) {
    let home_history = nested_u64(
        &prepared.data_quality,
        &["home", "team_features", "history_match_count"],
    )
    .unwrap_or(0);
    let away_history = nested_u64(
        &prepared.data_quality,
        &["away", "team_features", "history_match_count"],
    )
    .unwrap_or(0);
    let history_status = if home_history >= 5 && away_history >= 5 {
        PredictionReadinessCheckStatus::Passed
    } else {
        PredictionReadinessCheckStatus::Warning
    };
    let mut history_details = Vec::new();
    if home_history < 5 {
        history_details.push(format!(
            "{} 截止当前窗口只有 {home_history} 场有效历史比赛",
            prepared.match_record.home_team_name
        ));
    }
    if away_history < 5 {
        history_details.push(format!(
            "{} 截止当前窗口只有 {away_history} 场有效历史比赛",
            prepared.match_record.away_team_name
        ));
    }
    if home_history == 0 || away_history == 0 {
        shadow_reasons.push("球队历史样本存在零覆盖，当前输入只允许进入影子推演".to_string());
    }
    checks.push(readiness_check(
        ("team_history", "球队历史样本"),
        history_status,
        10,
        if history_status == PredictionReadinessCheckStatus::Passed {
            10
        } else if home_history == 0 || away_history == 0 {
            2
        } else {
            6
        },
        if history_status == PredictionReadinessCheckStatus::Passed {
            "双方均具备至少 5 场截止时点前的有效历史样本"
        } else {
            "球队历史样本不足，相关强度已按置信度回归中性"
        },
        history_details,
        json!({"home_history_match_count": home_history, "away_history_match_count": away_history}),
    ));

    let quality_score = prepared
        .match_input
        .get("feature_quality_score")
        .and_then(Value::as_f64)
        .unwrap_or(0.0)
        .clamp(0.0, 1.0);
    let quality_status = if quality_score >= 0.65 {
        PredictionReadinessCheckStatus::Passed
    } else {
        PredictionReadinessCheckStatus::Warning
    };
    let mut quality_details = Vec::new();
    if let Some(warning) = prepared
        .data_quality
        .get("warning")
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
    {
        quality_details.push(warning.to_string());
    }
    if quality_score < 0.40 {
        shadow_reasons.push(format!(
            "综合特征质量 {:.0}% 低于正式推演最低门槛 40%，当前仅允许影子推演",
            quality_score * 100.0
        ));
    }
    checks.push(readiness_check(
        ("model_input", "模型输入构建与质量"),
        quality_status,
        5,
        if quality_score >= 0.65 {
            5
        } else if quality_score >= 0.40 {
            3
        } else {
            1
        },
        if quality_score >= 0.65 {
            "确定性输入已生成，综合质量达到正式标准"
        } else if quality_score >= 0.40 {
            "确定性输入已生成，但综合质量需要在结果中保留警告"
        } else {
            "确定性输入已生成，但综合质量只适合影子验证"
        },
        quality_details,
        json!({
            "feature_quality_score": quality_score,
            "preparation_version": prepared.match_input.get("preparation_version"),
            "data_quality": &prepared.data_quality,
        }),
    ));
}

pub(crate) fn nested_u64(value: &Value, path: &[&str]) -> Option<u64> {
    let mut current = value;
    for key in path {
        current = current.get(*key)?;
    }
    current.as_u64()
}
