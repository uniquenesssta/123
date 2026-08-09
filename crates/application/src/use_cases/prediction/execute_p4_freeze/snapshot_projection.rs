use crate::use_cases::prediction::shared::p4_planning::canonical_fact_keys;
use crate::{ApplicationError, ApplicationResult};
use football_domain::{
    EvidenceVerificationState, P4FreezeReadiness, P4FreezeTaskRecord, P4RoutedFact, RouteDecision,
    SnapshotFeatureDraft, SnapshotProbabilityDraft,
};
use serde_json::{json, Map, Value};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};

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

pub(super) fn snapshot_features(
    task: &P4FreezeTaskRecord,
    data_quality: &Value,
    readiness: &P4FreezeReadiness,
    routes: &[P4RoutedFact],
) -> ApplicationResult<Vec<SnapshotFeatureDraft>> {
    let mut by_field = BTreeMap::<String, Vec<&P4RoutedFact>>::new();
    for route in routes {
        by_field
            .entry(route.field_key.clone())
            .or_default()
            .push(route);
    }
    let mut features = Vec::with_capacity(31);
    for (index, field_key) in canonical_fact_keys().into_iter().enumerate() {
        let field_routes = by_field.get(&field_key).cloned().unwrap_or_default();
        let mut evidence_ids = field_routes
            .iter()
            .flat_map(|route| route.selected_evidence_ids.iter().copied())
            .collect::<Vec<_>>();
        evidence_ids.sort_unstable();
        evidence_ids.dedup();
        let verification_state = aggregate_verification_state(&field_routes)?;
        let value = if field_routes.is_empty() {
            Value::Null
        } else if field_routes.len() == 1 {
            field_routes[0].selected_value.clone()
        } else {
            Value::Array(
                field_routes
                    .iter()
                    .map(|route| {
                        json!({
                            "route_key": route.route_key,
                            "entity_target": {
                                "module": route.target_module,
                                "slot": route.target_slot,
                            },
                            "value": route.selected_value,
                        })
                    })
                    .collect(),
            )
        };
        features.push(SnapshotFeatureDraft {
            field_order: u8::try_from(index + 1)
                .map_err(|_| ApplicationError::Validation("P4快照字段序号溢出".to_string()))?,
            field_key,
            value,
            verification_state,
            evidence_ids,
            metadata: json!({
                "orchestration_task_id": task.id,
                "route_count": field_routes.len(),
            }),
        });
    }
    features.push(SnapshotFeatureDraft {
        field_order: 30,
        field_key: "database_pre_match_features".to_string(),
        value: data_quality.clone(),
        verification_state: EvidenceVerificationState::NotApplicable,
        evidence_ids: Vec::new(),
        metadata: json!({"source": "PostgreSQL cutoff-aware deterministic preparation"}),
    });
    features.push(SnapshotFeatureDraft {
        field_order: 31,
        field_key: "orchestration_readiness".to_string(),
        value: serde_json::to_value(readiness)?,
        verification_state: EvidenceVerificationState::NotApplicable,
        evidence_ids: Vec::new(),
        metadata: json!({
            "contract_version": football_domain::P4_ORCHESTRATION_CONTRACT_VERSION,
            "state": "READY_TO_FREEZE",
        }),
    });
    Ok(features)
}

fn aggregate_verification_state(
    routes: &[&P4RoutedFact],
) -> ApplicationResult<EvidenceVerificationState> {
    if routes.is_empty() {
        return Ok(EvidenceVerificationState::NotFound);
    }
    let mut states = BTreeSet::new();
    for route in routes {
        states.insert(route.verification_state.as_str());
    }
    if states.contains("CONFLICT") || states.contains("STALE") {
        return Err(ApplicationError::Validation(
            "READY_TO_FREEZE任务仍包含CONFLICT或STALE路由".to_string(),
        ));
    }
    if states.contains("PROBABLE") {
        Ok(EvidenceVerificationState::Probable)
    } else if states.contains("CONFIRMED") {
        Ok(EvidenceVerificationState::Confirmed)
    } else if states.contains("NOT_FOUND") {
        Ok(EvidenceVerificationState::NotFound)
    } else {
        Ok(EvidenceVerificationState::NotApplicable)
    }
}

pub(super) fn snapshot_probabilities(
    payload: &Value,
) -> ApplicationResult<Vec<SnapshotProbabilityDraft>> {
    let matrices = payload
        .get("matrices")
        .and_then(Value::as_object)
        .ok_or_else(|| ApplicationError::Validation("外部模型输出缺少 matrices".to_string()))?;
    if matrices.is_empty() {
        return Err(ApplicationError::Validation(
            "外部模型输出至少需要一条概率矩阵".to_string(),
        ));
    }
    let clean_sheet_home = payload.get("clean_sheet_a").and_then(Value::as_f64);
    let clean_sheet_away = payload.get("clean_sheet_b").and_then(Value::as_f64);
    matrices
        .iter()
        .map(|(chain_key, matrix)| {
            let outcome = matrix
                .get("outcome")
                .and_then(Value::as_object)
                .ok_or_else(|| {
                    ApplicationError::Validation(format!("外部模型矩阵 {chain_key} 缺少 outcome"))
                })?;
            let scorelines = matrix
                .get("scorelines")
                .and_then(Value::as_array)
                .ok_or_else(|| {
                    ApplicationError::Validation(format!(
                        "外部模型矩阵 {chain_key} 缺少 scorelines"
                    ))
                })?;
            let matrix_cell_count = u16::try_from(scorelines.len()).map_err(|_| {
                ApplicationError::Validation(format!(
                    "外部模型矩阵 {chain_key} 的比分单元数量超出支持范围"
                ))
            })?;
            if matrix_cell_count == 0 {
                return Err(ApplicationError::Validation(format!(
                    "外部模型矩阵 {chain_key} 不得为空"
                )));
            }
            let is_formal = matrix
                .get("formal")
                .and_then(Value::as_bool)
                .unwrap_or(chain_key == "full");
            Ok(SnapshotProbabilityDraft {
                chain_key: chain_key.to_string(),
                home_win: required_probability(outcome, "a_win", chain_key)?,
                draw: required_probability(outcome, "draw", chain_key)?,
                away_win: required_probability(outcome, "b_win", chain_key)?,
                btts: matrix.get("btts").and_then(Value::as_f64),
                over_2_5: matrix.get("over_2_5").and_then(Value::as_f64),
                clean_sheet_home: is_formal.then_some(clean_sheet_home).flatten(),
                clean_sheet_away: is_formal.then_some(clean_sheet_away).flatten(),
                matrix_sha256: sha256_value(matrix)?,
                matrix_cell_count,
                metadata: json!({
                    "formal": is_formal,
                    "provider_owned_topology": true
                }),
            })
        })
        .collect()
}

fn required_probability(
    outcome: &Map<String, Value>,
    key: &str,
    chain_key: &str,
) -> ApplicationResult<f64> {
    outcome
        .get(key)
        .and_then(Value::as_f64)
        .filter(|value| value.is_finite() && (0.0..=1.0).contains(value))
        .ok_or_else(|| {
            ApplicationError::Validation(format!("外部模型矩阵 {chain_key} 的概率字段 {key} 无效"))
        })
}

fn sha256_value(value: &Value) -> ApplicationResult<String> {
    let bytes = serde_json::to_vec(value)?;
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    Ok(hex::encode(hasher.finalize()))
}
