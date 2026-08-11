use football_domain::{
    SpreadsheetAction, SpreadsheetEntityType, SpreadsheetImportPreview, SpreadsheetParsedWorkbook,
    TeamPackageCoverage,
};

pub(super) fn calculate(
    parsed: &SpreadsheetParsedWorkbook,
    team_preview: Option<&SpreadsheetImportPreview>,
    player_preview: Option<&SpreadsheetImportPreview>,
) -> TeamPackageCoverage {
    let count = |entity_type| {
        parsed
            .rows
            .iter()
            .filter(|row| row.entity_type == entity_type && row.action != SpreadsheetAction::Skip)
            .count() as u64
    };
    let team_count = count(SpreadsheetEntityType::Team);
    let player_count = count(SpreadsheetEntityType::Player);
    let coach_count = count(SpreadsheetEntityType::Coach);
    let formation_usage_count = count(SpreadsheetEntityType::FormationUsage);
    let team_ability_count = count(SpreadsheetEntityType::TeamAbilityObservation);
    let player_ability_count = count(SpreadsheetEntityType::PlayerAbility);
    let player_dynamic_tag_count = count(SpreadsheetEntityType::PlayerDynamicTag);
    let player_role_count = parsed
        .rows
        .iter()
        .filter(|row| {
            row.entity_type == SpreadsheetEntityType::PlayerPosition
                && row.action != SpreadsheetAction::Skip
                && row
                    .values
                    .get("default_role_code")
                    .and_then(serde_json::Value::as_str)
                    .is_some_and(|value| !value.trim().is_empty())
        })
        .count() as u64;
    let availability_count = count(SpreadsheetEntityType::PlayerAvailability);
    let preview_blocking = team_preview
        .map(|value| value.counts.conflict + value.counts.error)
        .unwrap_or_default()
        + player_preview
            .map(|value| value.counts.conflict + value.counts.error)
            .unwrap_or_default();
    let mut blockers = Vec::new();
    let mut warnings = Vec::new();
    if preview_blocking > 0 {
        blockers.push(format!(
            "完整资料包预检仍有 {preview_blocking} 条冲突或错误，修复后才能正式提交"
        ));
    }
    if team_count == 0 {
        blockers.push("缺少球队总览记录".to_string());
    }
    if player_count < 11 {
        blockers.push(format!(
            "有效球员只有 {player_count} 人，低于 P4 阵容输入下限 11 人"
        ));
    }
    if coach_count == 0 {
        warnings.push("没有教练记录，阵型只能回退到球队或系统默认".to_string());
    }
    if formation_usage_count == 0 {
        warnings.push("没有阵型使用分布，P4 将回退未知阵型".to_string());
    }
    if team_ability_count == 0 {
        warnings.push("没有球队能力观察".to_string());
    }
    if player_count > 0 && player_ability_count < player_count * 4 {
        warnings.push(format!(
            "球员能力观察只有 {player_ability_count} 条，建议至少每名球员填写 4 个核心维度"
        ));
    }
    if player_count > 0 && player_dynamic_tag_count < player_count * 3 {
        warnings.push(format!(
            "球员动态标签只有 {player_dynamic_tag_count} 条，建议至少覆盖准备度、状态和体能"
        ));
    }
    if player_count > 0 && player_role_count < player_count.min(11) {
        warnings.push(format!(
            "默认战术角色只有 {player_role_count} 条；未覆盖球员仍可保存阵容，但会降低角色输入完整度"
        ));
    }
    let player_factor = (player_count.min(26) as f64 / 26.0 * 20.0).round() as u8;
    let ability_target = (player_count * 8).max(1);
    let ability_factor = ((player_ability_count.min(ability_target) as f64 / ability_target as f64)
        * 25.0)
        .round() as u8;
    let tag_target = (player_count * 5).max(1);
    let tag_factor = ((player_dynamic_tag_count.min(tag_target) as f64 / tag_target as f64) * 20.0)
        .round() as u8;
    let mut readiness_score = 0_u8;
    if team_count > 0 {
        readiness_score += 15;
    }
    readiness_score += player_factor;
    if coach_count > 0 {
        readiness_score += 8;
    }
    if formation_usage_count > 0 {
        readiness_score += 7;
    }
    if team_ability_count > 0 {
        readiness_score += 5;
    }
    readiness_score += ability_factor;
    readiness_score += tag_factor;
    readiness_score = readiness_score.min(100);
    if preview_blocking > 0 {
        readiness_score = readiness_score.min(69);
    }
    TeamPackageCoverage {
        team_count,
        player_count,
        coach_count,
        formation_usage_count,
        team_ability_count,
        player_ability_count,
        player_dynamic_tag_count,
        player_role_count,
        availability_count,
        readiness_score,
        p4_input_ready: blockers.is_empty() && preview_blocking == 0 && readiness_score >= 70,
        blockers,
        warnings,
    }
}
