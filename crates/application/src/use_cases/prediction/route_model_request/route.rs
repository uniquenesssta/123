use crate::{ApplicationError, ApplicationResult};
use football_domain::{RouteDecision, RuleRouting};
use serde_json::{json, Value};

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
