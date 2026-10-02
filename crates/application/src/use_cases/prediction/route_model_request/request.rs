use super::selection::normalize_model_selection;
use crate::model_shell::P4_MODEL_ID;
use crate::{
    p4_default_match, p4_default_parameters, ApplicationError, ApplicationResult, PredictionCommand,
};
use chrono::{DateTime, Utc};
use football_domain::{
    CompetitionKind, MatchContext, ModelIdentity, ResolvedCompetitionContext, RouteDecision,
};
use football_model_api::ModelRequest;
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

// Explicit rule packages retain the existing model context override.
pub(crate) fn apply_route_context(
    command: &PredictionCommand,
    context: &mut MatchContext,
    scope: &ResolvedCompetitionContext,
    decision: &RouteDecision,
) -> ApplicationResult<()> {
    if command.explicit_rule_package_id.is_some() {
        context.competition_kind = decision.competition_profile.competition_kind;
        if let Some(metadata) = context.metadata.as_object_mut() {
            metadata.insert(
                "explicit_competition_kind_override".to_string(),
                json!({
                    "catalog_kind": scope.competition_kind.as_str(),
                    "rule_package_kind": decision.competition_profile.competition_kind.as_str(),
                }),
            );
        }
    } else if decision.competition_profile.competition_kind != scope.competition_kind {
        return Err(ApplicationError::Validation(format!(
            "自动规则包赛事类型 {} 与当前赛事类型 {} 不一致",
            decision.competition_profile.competition_kind.as_str(),
            scope.competition_kind.as_str()
        )));
    }
    Ok(())
}

pub(crate) fn build_model_request(
    command: PredictionCommand,
    context: MatchContext,
    decision: &RouteDecision,
) -> ApplicationResult<ModelRequest> {
    let match_input = ensure_match_input_id(command.match_input, &context.match_key)?;
    let request = ModelRequest {
        context,
        identity: ModelIdentity {
            model_id: decision.model_id.clone(),
            model_version: decision.model_version.clone(),
            parameter_version: decision.parameter_version.clone(),
            rule_package_version: Some(decision.package_version.clone()),
        },
        snapshot_type: command.snapshot_type,
        input: match_input,
        parameters: decision.parameters.clone(),
    };
    Ok(request)
}

pub(crate) fn default_fixture_request() -> ApplicationResult<ModelRequest> {
    let parameters = p4_default_parameters();
    let match_input = p4_default_match();
    let request = ModelRequest {
        context: MatchContext {
            match_key: required_string(&match_input, "match_id")?,
            kickoff_time: parse_kickoff(&required_string(&match_input, "kickoff_time")?)?,
            competition_id: None,
            season_id: None,
            stage_id: None,
            competition_kind: CompetitionKind::Custom,
            home_team_name: nested_required_string(&match_input, "team_a", "name")?,
            away_team_name: nested_required_string(&match_input, "team_b", "name")?,
            metadata: Value::Null,
        },
        identity: ModelIdentity {
            model_id: P4_MODEL_ID.to_string(),
            model_version: required_string(&parameters, "model_version")?,
            parameter_version: required_string(&parameters, "parameter_version")?,
            rule_package_version: None,
        },
        snapshot_type: "T-1h".to_string(),
        input: match_input,
        parameters,
    };
    Ok(request)
}
