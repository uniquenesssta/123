use crate::model_shell::P4_MODEL_ID;
use crate::{ApplicationError, ApplicationResult};
use football_domain::{P4FreezeTaskRecord, P4Horizon, RouteDecision};
use std::collections::BTreeSet;
use uuid::Uuid;

pub(crate) fn validate_requested_fact_keys(
    requested: Vec<String>,
) -> ApplicationResult<Vec<String>> {
    let canonical = canonical_fact_keys();
    if requested.is_empty() {
        return Ok(canonical);
    }
    let requested_set = requested
        .into_iter()
        .map(|value| value.trim().to_string())
        .collect::<BTreeSet<_>>();
    let canonical_set = canonical.iter().cloned().collect::<BTreeSet<_>>();
    if requested_set != canonical_set {
        return Err(ApplicationError::Validation(
            "正式P4冻结必须研究路由注册表中的全部29个事实字段；不得以子集生成31字段正式快照"
                .to_string(),
        ));
    }
    Ok(canonical)
}

pub(crate) fn canonical_fact_keys() -> Vec<String> {
    let registry: football_domain::EvidenceRouteRegistry =
        serde_json::from_str(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../src-tauri/resources/research/public_evidence_routes.json"
        )))
        .expect("内置P4证据路由注册表必须有效");
    registry
        .routes
        .into_iter()
        .map(|route| route.field_key)
        .collect()
}

pub(crate) fn is_p4_model(model_id: &str) -> bool {
    model_id == P4_MODEL_ID || model_id.starts_with("p4_")
}

pub(crate) fn horizon_priority(horizon: P4Horizon) -> i32 {
    match horizon {
        P4Horizon::T24h => 10,
        P4Horizon::T6h => 20,
        P4Horizon::T90m => 30,
        P4Horizon::T1h => 40,
        P4Horizon::LegacyTN => 0,
    }
}

pub(crate) fn validate_existing_task_identity(
    task: &P4FreezeTaskRecord,
    route: &RouteDecision,
    research_schema_version_id: Uuid,
    snapshot_schema_version_id: Uuid,
    requested_fact_keys: &[String],
) -> ApplicationResult<()> {
    let existing_facts = task
        .requested_fact_keys
        .iter()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    let requested_facts = requested_fact_keys
        .iter()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    if task.rule_package_id != route.rule_package_id
        || task.model_version_id != route.model_version_id
        || task.parameter_set_id != route.parameter_set_id
        || task.competition_profile_id != route.competition_profile_id
        || task.research_schema_version_id != research_schema_version_id
        || task.snapshot_schema_version_id != snapshot_schema_version_id
        || existing_facts != requested_facts
    {
        return Err(ApplicationError::Validation(
            "同一正式队列键已存在，但规则包、模型、参数、赛事Profile、Schema或事实字段与当前规划请求不一致"
                .to_string(),
        ));
    }
    Ok(())
}
