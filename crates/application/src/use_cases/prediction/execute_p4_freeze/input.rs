use crate::{ApplicationError, ApplicationResult};
use football_domain::{P4FreezeReadiness, P4FreezeTaskRecord, P4RoutedFact, RouteDecision};
use serde_json::{json, Value};

pub(super) fn attach_orchestration_input(
    match_input: &mut Value,
    task: &P4FreezeTaskRecord,
    readiness: &P4FreezeReadiness,
    routed_facts: &[P4RoutedFact],
) -> ApplicationResult<()> {
    let object = match_input.as_object_mut().ok_or_else(|| {
        ApplicationError::Validation("数据库构建的P4输入不是JSON对象".to_string())
    })?;
    object.insert(
        "p4_orchestration".to_string(),
        json!({
            "contract_version": football_domain::P4_ORCHESTRATION_CONTRACT_VERSION,
            "task_id": task.id,
            "trace_id": task.trace_id,
            "horizon": task.horizon.as_str(),
            "data_cutoff_at": task.data_cutoff_at,
            "readiness": readiness,
            "routed_facts": routed_facts,
            "numeric_transform_policy": "freeze_provenance_without_unversioned_weight_invention"
        }),
    );
    Ok(())
}

pub(super) fn validate_pinned_route(
    task: &P4FreezeTaskRecord,
    route: &RouteDecision,
) -> ApplicationResult<()> {
    if route.rule_package_id != task.rule_package_id
        || route.model_version_id != task.model_version_id
        || route.parameter_set_id != task.parameter_set_id
        || route.competition_profile_id != task.competition_profile_id
    {
        return Err(ApplicationError::Validation(
            "冻结执行时的规则包、模型、参数或赛事Profile与规划时锁定身份不一致".to_string(),
        ));
    }
    Ok(())
}
