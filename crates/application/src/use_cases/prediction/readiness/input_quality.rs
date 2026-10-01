use super::report::readiness_check;
use football_domain::{PredictionReadinessCheck, PredictionReadinessCheckStatus};
use serde_json::{json, Value};

pub(super) fn append_unavailable_prepared_input_checks(
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

pub(super) fn append_prepared_input_checks(
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

pub(super) fn nested_u64(value: &Value, path: &[&str]) -> Option<u64> {
    let mut current = value;
    for key in path {
        current = current.get(*key)?;
    }
    current.as_u64()
}
