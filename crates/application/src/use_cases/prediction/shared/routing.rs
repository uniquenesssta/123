use crate::{
    model_registry::ModelRegistry, ApplicationError, ApplicationResult, PredictionCommand,
};
use chrono::{DateTime, Utc};
use football_domain::{MatchContext, ResolvedCompetitionContext, RouteDecision, RuleRouting};
use serde_json::{json, Value};

pub(crate) fn match_context_from_command(
    command: &PredictionCommand,
    scope: &ResolvedCompetitionContext,
) -> ApplicationResult<MatchContext> {
    let kickoff_time = parse_kickoff(&required_string(&command.match_input, "kickoff_time")?)?;
    let home_team_name = nested_required_string(&command.match_input, "team_a", "name")?;
    let away_team_name = nested_required_string(&command.match_input, "team_b", "name")?;
    let match_key = command
        .match_input
        .get("match_id")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .unwrap_or_else(|| {
            format!(
                "SIM-{}-{}-{}",
                kickoff_time.format("%Y%m%dT%H%MZ"),
                compact_key_part(&home_team_name),
                compact_key_part(&away_team_name)
            )
        });
    let model_selection = normalize_model_selection(&command.model_family)?;
    Ok(MatchContext {
        match_key,
        kickoff_time,
        competition_id: scope.competition_id,
        season_id: scope.season_id,
        stage_id: scope.stage_id,
        competition_kind: scope.competition_kind,
        home_team_name,
        away_team_name,
        metadata: json!({
            "routing_mode": if command.explicit_rule_package_id.is_some() { "explicit_rule_package" } else { "automatic" },
            "requested_model_family": model_selection.family,
            "requested_model_id": model_selection.exact_model_id,
        }),
    })
}

pub(crate) fn ensure_match_input_id(mut input: Value, match_key: &str) -> ApplicationResult<Value> {
    let object = input
        .as_object_mut()
        .ok_or_else(|| ApplicationError::Model("模型输入必须是 JSON 对象".to_string()))?;
    let has_match_id = object
        .get("match_id")
        .and_then(Value::as_str)
        .is_some_and(|value| !value.trim().is_empty());
    if !has_match_id {
        object.insert("match_id".to_string(), Value::String(match_key.to_string()));
    }
    Ok(input)
}

pub(crate) fn compact_key_part(value: &str) -> String {
    let normalized: String = value
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric())
        .map(|ch| ch.to_ascii_uppercase())
        .take(12)
        .collect();
    if normalized.is_empty() {
        "TEAM".to_string()
    } else {
        normalized
    }
}

pub(crate) fn validate_snapshot_type(
    snapshot_type: &str,
    routing: &RuleRouting,
) -> ApplicationResult<()> {
    if snapshot_type.trim().is_empty() {
        return Err(ApplicationError::Validation("快照类型不能为空".to_string()));
    }
    if !routing.supported_snapshot_types.is_empty()
        && !routing
            .supported_snapshot_types
            .iter()
            .any(|item| item == snapshot_type)
    {
        return Err(ApplicationError::Validation(format!(
            "规则包不支持快照类型 {snapshot_type}"
        )));
    }
    Ok(())
}

pub(crate) fn route_identity_manifest(decision: &RouteDecision) -> Value {
    json!({
        "source": decision.source,
        "binding_id": decision.binding_id,
        "rule_package_id": decision.rule_package_id,
        "rule_package_key": decision.package_key,
        "rule_package_version": decision.package_version,
        "model_id": decision.model_id,
        "model_version_id": decision.model_version_id,
        "model_version": decision.model_version,
        "parameter_set_id": decision.parameter_set_id,
        "parameter_version": decision.parameter_version,
        "competition_profile_id": decision.competition_profile_id,
    })
}

pub(crate) fn verify_route_identity_matches_input_audit(
    decision: &RouteDecision,
    input: &Value,
) -> ApplicationResult<()> {
    let Some(expected) = input
        .get("input_audit")
        .and_then(|audit| audit.get("manifest"))
        .and_then(|manifest| manifest.get("route_identity"))
    else {
        return Ok(());
    };
    let actual = route_identity_manifest(decision);
    if expected != &actual {
        return Err(ApplicationError::Validation(
            "模型、参数或规则路由在完整度检查后发生变化，请重新检查后再运行".to_string(),
        ));
    }
    Ok(())
}

#[derive(Debug, Clone)]
pub(crate) struct ModelSelection {
    pub(crate) family: &'static str,
    pub(crate) exact_model_id: Option<String>,
}

pub(crate) fn normalize_model_selection(value: &str) -> ApplicationResult<ModelSelection> {
    let normalized = value.trim().to_ascii_lowercase();
    let normalized = if normalized.is_empty() {
        "p4".to_string()
    } else {
        normalized
    };
    let family = if normalized == "p4" || normalized.starts_with("p4_") {
        "p4"
    } else if normalized == "p7" || normalized.starts_with("p7_") {
        "p7"
    } else {
        return Err(ApplicationError::Validation(format!(
            "不支持的模型：{normalized}；请选择已注册的 P4 或 P7 模型"
        )));
    };
    let exact_model_id = if normalized == family {
        None
    } else {
        Some(normalized)
    };
    Ok(ModelSelection {
        family,
        exact_model_id,
    })
}

pub(crate) fn ensure_model_selection_registered(
    registry: &ModelRegistry,
    selection: &ModelSelection,
) -> ApplicationResult<()> {
    if let Some(model_id) = selection.exact_model_id.as_deref() {
        if registry.get(model_id).is_none() {
            return Err(ApplicationError::ModelNotFound(model_id.to_string()));
        }
    }
    Ok(())
}

pub(crate) fn parse_kickoff(raw: &str) -> ApplicationResult<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(raw)
        .map_err(|error| ApplicationError::InvalidKickoff(error.to_string()))
        .map(|value| value.with_timezone(&Utc))
}

pub(crate) fn required_string(value: &Value, key: &str) -> ApplicationResult<String> {
    value
        .get(key)
        .and_then(Value::as_str)
        .map(str::to_string)
        .filter(|item| !item.trim().is_empty())
        .ok_or_else(|| ApplicationError::Model(format!("缺少字符串字段：{key}")))
}

pub(crate) fn nested_required_string(
    value: &Value,
    parent: &str,
    key: &str,
) -> ApplicationResult<String> {
    value
        .get(parent)
        .and_then(|parent_value| parent_value.get(key))
        .and_then(Value::as_str)
        .map(str::to_string)
        .filter(|item| !item.trim().is_empty())
        .ok_or_else(|| ApplicationError::Model(format!("缺少字符串字段：{parent}.{key}")))
}
