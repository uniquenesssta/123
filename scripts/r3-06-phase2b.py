from pathlib import Path
import json

ROOT = Path(__file__).resolve().parents[1]


def read(path: str) -> str:
    return (ROOT / path).read_text(encoding="utf-8")


def write(path: str, text: str) -> None:
    target = ROOT / path
    target.parent.mkdir(parents=True, exist_ok=True)
    target.write_text(text, encoding="utf-8", newline="\n")


def replace_once(path: str, old: str, new: str) -> None:
    text = read(path)
    if text.count(old) != 1:
        raise RuntimeError(f"expected exactly one match in {path}: {old[:120]!r}")
    write(path, text.replace(old, new, 1))


# 1. Prediction Port: add a cohesive freeze-persistence boundary.
ports_path = "crates/application/src/ports/prediction/mod.rs"
ports = read(ports_path)
ports = ports.replace(
    "    P4FreezeTaskTransition, P4MatchWorkspace, P4PlanningMatchContext, P4TaskWorkspace,\n    PredictionSummary, PreparedMatchPredictionInput, RouteDecision,\n",
    "    P4FreezeTaskTransition, P4MatchWorkspace, P4PlanningMatchContext, P4RoutedFact,\n    P4TaskWorkspace, PredictionSummary, PrematchSnapshotDraft, PrematchSnapshotRecord,\n    PreparedMatchPredictionInput, RouteDecision,\n",
    1,
)
if "pub trait P4FreezeExecutionPort" in ports:
    raise RuntimeError("P4FreezeExecutionPort already exists")
ports += """

#[async_trait]
pub trait P4FreezeExecutionPort: Send + Sync {
    async fn find_frozen_snapshot_id(
        &self,
        task: &P4FreezeTaskRecord,
    ) -> PortResult<Option<Uuid>>;
    async fn routed_facts(&self, task_id: Uuid) -> PortResult<Vec<P4RoutedFact>>;
    async fn freeze_snapshot(
        &self,
        draft: &PrematchSnapshotDraft,
    ) -> PortResult<PrematchSnapshotRecord>;
}
"""
write(ports_path, ports)

# Keep the machine-readable Port inventory authoritative.
inventory_path = ROOT / "architecture/application-port-inventory.json"
inventory = json.loads(inventory_path.read_text(encoding="utf-8"))
prediction_domain = next(item for item in inventory["domains"] if item["name"] == "prediction")
if "P4FreezeExecutionPort" not in prediction_domain["traits"]:
    prediction_domain["traits"].append("P4FreezeExecutionPort")
inventory_path.write_text(json.dumps(inventory, ensure_ascii=False, indent=2) + "\n", encoding="utf-8", newline="\n")

# 2. Concrete adapter remains in composition root.
adapter_path = "crates/application/src/composition/adapters/prediction.rs"
adapter = read(adapter_path)
adapter = adapter.replace(
    "        ModelRunHistoryItem, ModelRunPort, PredictionInputPort, PredictionWorkflowPort,\n        SerializedModelRun,\n",
    "        ModelRunHistoryItem, ModelRunPort, P4FreezeExecutionPort, PredictionInputPort,\n        PredictionWorkflowPort, SerializedModelRun,\n",
    1,
)
adapter = adapter.replace(
    "    P4FreezeReadiness, P4FreezeTaskDraft, P4FreezeTaskEventRecord, P4FreezeTaskRecord,\n    P4FreezeTaskTransition, P4MatchWorkspace, P4PlanningMatchContext, P4TaskWorkspace,\n    PredictionSummary, PreparedMatchPredictionInput, RouteDecision,\n",
    "    P4FreezeReadiness, P4FreezeTaskDraft, P4FreezeTaskEventRecord, P4FreezeTaskRecord,\n    P4FreezeTaskTransition, P4MatchWorkspace, P4PlanningMatchContext, P4RoutedFact,\n    P4TaskWorkspace, PredictionSummary, PrematchSnapshotDraft, PrematchSnapshotRecord,\n    PreparedMatchPredictionInput, RouteDecision,\n",
    1,
)
if "impl P4FreezeExecutionPort for ActiveDatabase" in adapter:
    raise RuntimeError("freeze adapter already exists")
adapter += """

#[async_trait]
impl P4FreezeExecutionPort for ActiveDatabase {
    async fn find_frozen_snapshot_id(
        &self,
        task: &P4FreezeTaskRecord,
    ) -> PortResult<Option<Uuid>> {
        self.transition_store()
            .find_frozen_p4_snapshot_id(task)
            .await
            .map_err(map_persistence_error)
    }

    async fn routed_facts(&self, task_id: Uuid) -> PortResult<Vec<P4RoutedFact>> {
        self.transition_store()
            .p4_routed_facts(task_id)
            .await
            .map_err(map_persistence_error)
    }

    async fn freeze_snapshot(
        &self,
        draft: &PrematchSnapshotDraft,
    ) -> PortResult<PrematchSnapshotRecord> {
        self.transition_store()
            .freeze_prematch_snapshot(draft)
            .await
            .map_err(map_persistence_error)
    }
}
"""
write(adapter_path, adapter)

# 3. Use-case access boundary and module registration.
use_mod_path = "crates/application/src/use_cases/prediction/mod.rs"
use_mod = read(use_mod_path)
use_mod = use_mod.replace(
    "    prediction::{ModelRunPort, PredictionInputPort},\n",
    "    prediction::{ModelRunPort, P4FreezeExecutionPort, PredictionInputPort},\n",
    1,
)
use_mod = use_mod.replace(
    "pub(crate) mod execute_prediction;\n",
    "pub(crate) mod execute_p4_freeze;\npub(crate) mod execute_prediction;\n",
    1,
)
use_mod += """

pub(crate) trait P4FreezeExecutionAccess:
    PredictionAccess
    + crate::ports::prediction::PredictionWorkflowPort
    + P4FreezeExecutionPort
    + crate::ports::research::ResearchArtifactPort
{
}

impl<T> P4FreezeExecutionAccess for T where
    T: PredictionAccess
        + crate::ports::prediction::PredictionWorkflowPort
        + P4FreezeExecutionPort
        + crate::ports::research::ResearchArtifactPort
{
}
"""
write(use_mod_path, use_mod)

# 4. Freeze snapshot projection owns deterministic 31-field / probability translation.
write(
    "crates/application/src/use_cases/prediction/execute_p4_freeze/snapshot_projection.rs",
    r'''use crate::use_cases::prediction::shared::p4_planning::canonical_fact_keys;
use crate::{ApplicationError, ApplicationResult};
use football_domain::{
    EvidenceVerificationState, P4FreezeReadiness, P4FreezeTaskRecord, P4RoutedFact,
    RouteDecision, SnapshotFeatureDraft, SnapshotProbabilityDraft,
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
''',
)

# 5. Freeze execution orchestration: state machine + existing Prediction execution boundary.
write(
    "crates/application/src/use_cases/prediction/execute_p4_freeze/mod.rs",
    r'''mod snapshot_projection;

use self::snapshot_projection::{
    attach_orchestration_input, snapshot_features, snapshot_probabilities, validate_pinned_route,
};
use super::{execute_prediction, P4FreezeExecutionAccess};
use crate::built_in_artifacts::{
    P4_SNAPSHOT_SCHEMA_ARTIFACT_VERSION as SNAPSHOT_SCHEMA_VERSION,
    P4_SNAPSHOT_SCHEMA_KEY as SNAPSHOT_SCHEMA_KEY,
};
use crate::model_registry::ModelRegistry;
use crate::{ApplicationError, ApplicationResult, PredictionCommand};
use chrono::Utc;
use football_domain::{
    P4FreezeTaskRecord, P4FreezeTaskState, P4FreezeTaskTransition, PrematchSnapshotDraft,
    SnapshotSourceKind,
};
use serde_json::{json, Value};
use uuid::Uuid;

pub(crate) async fn execute<P: P4FreezeExecutionAccess + ?Sized>(
    port: &P,
    registry: &ModelRegistry,
    task_id: Uuid,
    job_id: Uuid,
) -> ApplicationResult<Value> {
    let mut task = port.read_freeze_task(task_id).await?;
    if task.state.is_terminal() {
        return Ok(json!({"task_id": task.id, "state": task.state, "noop": true}));
    }
    if let Some(snapshot_id) = port.find_frozen_snapshot_id(&task).await? {
        if task.state != P4FreezeTaskState::Freezing {
            return Err(ApplicationError::Validation(format!(
                "已存在不可变快照，但任务状态不是FREEZING：{}",
                task.state.as_str()
            )));
        }
        let frozen = port
            .transition_freeze_task(
                task.id,
                &P4FreezeTaskTransition {
                    task_id: task.id,
                    expected_state: P4FreezeTaskState::Freezing,
                    next_state: P4FreezeTaskState::Frozen,
                    reason: "恢复已写入但尚未登记FROZEN状态的不可变快照".to_string(),
                    blockers: json!([]),
                    payload: json!({
                        "snapshot_id": snapshot_id,
                        "job_id": job_id,
                        "recovered": true,
                    }),
                    research_run_id: None,
                    research_job_id: None,
                    freeze_job_id: None,
                    snapshot_id: Some(snapshot_id),
                },
            )
            .await?;
        return Ok(json!({
            "task_id": frozen.id,
            "state": frozen.state,
            "snapshot_id": snapshot_id,
            "recovered": true,
        }));
    }

    let now = Utc::now();
    if now < task.data_cutoff_at {
        return Err(ApplicationError::Validation(format!(
            "冻结任务被提前领取：截止时间 {}",
            task.data_cutoff_at.to_rfc3339()
        )));
    }
    if now > task.freeze_deadline_at {
        task = transition_missed(
            port,
            &task,
            "超过正式冻结宽限窗口，禁止用更晚事实回填",
            json!({"job_id": job_id, "now": now}),
        )
        .await?;
        return Ok(json!({"task_id": task.id, "state": task.state}));
    }

    let readiness = port.freeze_readiness(task.id).await?;
    if !readiness.ready {
        let blocked = port
            .transition_freeze_task(
                task.id,
                &P4FreezeTaskTransition {
                    task_id: task.id,
                    expected_state: task.state,
                    next_state: P4FreezeTaskState::Blocked,
                    reason: "冻结前复核未通过READY_TO_FREEZE门禁".to_string(),
                    blockers: serde_json::to_value(&readiness.blockers)?,
                    payload: serde_json::to_value(&readiness)?,
                    research_run_id: None,
                    research_job_id: None,
                    freeze_job_id: None,
                    snapshot_id: None,
                },
            )
            .await?;
        return Ok(json!({"task_id": blocked.id, "state": blocked.state}));
    }

    if task.state == P4FreezeTaskState::ResearchSucceeded {
        task = port
            .transition_freeze_task(
                task.id,
                &P4FreezeTaskTransition {
                    task_id: task.id,
                    expected_state: P4FreezeTaskState::ResearchSucceeded,
                    next_state: P4FreezeTaskState::ReadyToFreeze,
                    reason: "恢复已创建冻结Job但尚未登记READY_TO_FREEZE的任务".to_string(),
                    blockers: json!([]),
                    payload: json!({"job_id": job_id, "recovered": true}),
                    research_run_id: None,
                    research_job_id: None,
                    freeze_job_id: Some(job_id),
                    snapshot_id: None,
                },
            )
            .await?;
    }
    if task.state == P4FreezeTaskState::ReadyToFreeze {
        task = port
            .transition_freeze_task(
                task.id,
                &P4FreezeTaskTransition {
                    task_id: task.id,
                    expected_state: P4FreezeTaskState::ReadyToFreeze,
                    next_state: P4FreezeTaskState::Freezing,
                    reason: "到达正式数据截止时点，开始生成确定性P4快照".to_string(),
                    blockers: Value::Null,
                    payload: json!({"job_id": job_id, "frozen_at": now}),
                    research_run_id: None,
                    research_job_id: None,
                    freeze_job_id: None,
                    snapshot_id: None,
                },
            )
            .await?;
    }
    if task.state != P4FreezeTaskState::Freezing {
        return Err(ApplicationError::Validation(format!(
            "冻结任务状态不是FREEZING：{}",
            task.state.as_str()
        )));
    }

    let routed_facts = port.routed_facts(task.id).await?;
    let prepared = port
        .prepare_match_input(task.match_id, task.horizon.as_str(), "p4")
        .await?;
    let mut match_input = prepared.match_input;
    attach_orchestration_input(&mut match_input, &task, &readiness, &routed_facts)?;
    let execution = execute_prediction::execute(
        port,
        registry,
        PredictionCommand {
            match_input: match_input.clone(),
            snapshot_type: task.horizon.as_str().to_string(),
            competition_id: prepared.match_record.competition_id,
            season_id: prepared.match_record.season_id,
            stage_id: prepared.match_record.stage_id,
            competition_kind: prepared.competition_kind,
            model_family: "p4".to_string(),
            explicit_rule_package_id: Some(task.rule_package_id),
        },
    )
    .await?;
    validate_pinned_route(&task, &execution.route)?;

    let probabilities = snapshot_probabilities(&execution.output.payload)?;
    let features = snapshot_features(&task, &prepared.data_quality, &readiness, &routed_facts)?;
    let database_quality = match_input
        .get("feature_quality_score")
        .and_then(Value::as_f64)
        .unwrap_or(0.0);
    let route_quality = (readiness.routed_fact_count as f64
        / readiness.requested_fact_count.max(1) as f64)
        .clamp(0.0, 1.0);
    let quality_score = ((database_quality + route_quality) / 2.0).clamp(0.0, 1.0);

    let snapshot_schema = port
        .read_schema(SNAPSHOT_SCHEMA_KEY, SNAPSHOT_SCHEMA_VERSION)
        .await?;
    if snapshot_schema.id != task.snapshot_schema_version_id {
        return Err(ApplicationError::Validation(
            "冻结任务锁定的快照Schema版本发生变化".to_string(),
        ));
    }

    let frozen_at = Utc::now();
    if frozen_at > task.freeze_deadline_at {
        let missed = transition_missed(
            port,
            &task,
            "P4执行完成时已超过正式冻结宽限窗口，拒绝写入快照",
            json!({"job_id": job_id, "now": frozen_at}),
        )
        .await?;
        return Ok(json!({"task_id": missed.id, "state": missed.state}));
    }

    let snapshot = port
        .freeze_snapshot(&PrematchSnapshotDraft {
            match_id: task.match_id,
            match_key: task.match_key.clone(),
            horizon: task.horizon,
            data_cutoff_at: task.data_cutoff_at,
            frozen_at,
            model_version_id: task.model_version_id,
            parameter_set_id: task.parameter_set_id,
            competition_profile_id: task.competition_profile_id,
            research_run_id: task.research_run_id,
            schema_version_id: task.snapshot_schema_version_id,
            schema_version: SNAPSHOT_SCHEMA_VERSION.to_string(),
            trace_id: task.trace_id,
            idempotency_key: format!("p4-prematch-snapshot:{}", task.id),
            source_kind: SnapshotSourceKind::Real,
            quality_score,
            input_payload: json!({
                "model_run_id": execution.run_id,
                "match_input": match_input,
                "p4_output": execution.output.payload,
            }),
            features,
            probabilities,
            metadata: json!({
                "orchestration_contract": football_domain::P4_ORCHESTRATION_CONTRACT_VERSION,
                "orchestration_task_id": task.id,
                "research_run_id": task.research_run_id,
                "rule_package_id": task.rule_package_id,
                "model_run_id": execution.run_id,
                "readiness": readiness,
            }),
        })
        .await?;
    let frozen = port
        .transition_freeze_task(
            task.id,
            &P4FreezeTaskTransition {
                task_id: task.id,
                expected_state: P4FreezeTaskState::Freezing,
                next_state: P4FreezeTaskState::Frozen,
                reason: "31字段与四链概率已写入不可变正式快照".to_string(),
                blockers: json!([]),
                payload: json!({
                    "snapshot_id": snapshot.id,
                    "model_run_id": execution.run_id,
                    "quality_score": quality_score,
                }),
                research_run_id: None,
                research_job_id: None,
                freeze_job_id: None,
                snapshot_id: Some(snapshot.id),
            },
        )
        .await?;
    Ok(json!({
        "task_id": frozen.id,
        "state": frozen.state,
        "snapshot_id": snapshot.id,
        "model_run_id": execution.run_id,
    }))
}

async fn transition_missed<P: P4FreezeExecutionAccess + ?Sized>(
    port: &P,
    task: &P4FreezeTaskRecord,
    reason: &str,
    payload: Value,
) -> ApplicationResult<P4FreezeTaskRecord> {
    let next = P4FreezeTaskState::Missed;
    if !task.state.can_transition_to(next) {
        return Err(ApplicationError::Validation(format!(
            "任务状态 {} 不能转为MISSED",
            task.state.as_str()
        )));
    }
    Ok(port
        .transition_freeze_task(
            task.id,
            &P4FreezeTaskTransition {
                task_id: task.id,
                expected_state: task.state,
                next_state: next,
                reason: reason.to_string(),
                blockers: json!([reason]),
                payload,
                research_run_id: None,
                research_job_id: None,
                freeze_job_id: None,
                snapshot_id: None,
            },
        )
        .await?)
}
''',
)

# 6. Prediction Service / compatibility facade delegate to the new use case.
service_path = "crates/application/src/services/prediction/service.rs"
service = read(service_path)
service = service.replace(
    "    dry_run_default_fixture, execute_prediction, execute_prediction_from_match,\n",
    "    dry_run_default_fixture, execute_p4_freeze, execute_prediction, execute_prediction_from_match,\n",
    1,
)
service = service.replace(
    "    P4PlanningAccess, PredictionAccess,\n",
    "    P4FreezeExecutionAccess, P4PlanningAccess, PredictionAccess,\n",
    1,
)
marker = "    pub(crate) async fn plan_p4_horizons<P: P4PlanningAccess + ?Sized>(\n"
if service.count(marker) != 1:
    raise RuntimeError("PredictionService insertion marker mismatch")
method = """    pub(crate) async fn execute_p4_freeze_task<P: P4FreezeExecutionAccess + ?Sized>(
        &self,
        port: &P,
        registry: &ModelRegistry,
        task_id: Uuid,
        job_id: Uuid,
    ) -> ApplicationResult<Value> {
        execute_p4_freeze::execute(port, registry, task_id, job_id).await
    }

"""
service = service.replace(marker, method + marker, 1)
write(service_path, service)

facade_path = "crates/application/src/services/prediction/facade.rs"
facade = read(facade_path)
marker = "    pub async fn plan_p4_horizons(\n"
if facade.count(marker) != 1:
    raise RuntimeError("Prediction facade insertion marker mismatch")
method = """    pub(crate) async fn execute_p4_freeze_task(
        &self,
        task_id: Uuid,
        job_id: Uuid,
    ) -> ApplicationResult<Value> {
        let session = self.prediction_session().await?;
        self.prediction
            .execute_p4_freeze_task(&session, &self.registry, task_id, job_id)
            .await
    }

"""
facade = facade.replace(marker, method + marker, 1)
write(facade_path, facade)

# 7. Legacy mixed worker keeps Research ownership, delegates only the freeze branch.
legacy_path = "crates/application/src/p4_orchestration.rs"
legacy = read(legacy_path)
legacy = legacy.replace(
    "use super::{\n    ApplicationError, ApplicationResult, ApplicationService, OpenAiResearchCommand,\n    PredictionCommand,\n};\nuse crate::built_in_artifacts::{\n    P4_SNAPSHOT_SCHEMA_ARTIFACT_VERSION as SNAPSHOT_SCHEMA_VERSION,\n    P4_SNAPSHOT_SCHEMA_KEY as SNAPSHOT_SCHEMA_KEY,\n};\nuse crate::use_cases::prediction::shared::p4_planning::{canonical_fact_keys, horizon_priority};\n",
    "use super::{ApplicationError, ApplicationResult, ApplicationService, OpenAiResearchCommand};\nuse crate::use_cases::prediction::shared::p4_planning::horizon_priority;\n",
    1,
)
legacy = legacy.replace(
    "    EnqueueJobDraft, EvidenceVerificationState, P4FreezeReadiness, P4FreezeTaskRecord,\n    P4FreezeTaskState, P4FreezeTaskTransition, P4RoutedFact, PrematchSnapshotDraft,\n    ResearchRunDraft, ResearchRunStatus, SnapshotFeatureDraft, SnapshotProbabilityDraft,\n    SnapshotSourceKind, P4_ORCHESTRATION_PLANNER_VERSION,\n",
    "    EnqueueJobDraft, P4FreezeTaskRecord, P4FreezeTaskState, P4FreezeTaskTransition,\n    ResearchRunDraft, ResearchRunStatus, P4_ORCHESTRATION_PLANNER_VERSION,\n",
    1,
)
legacy = legacy.replace("use serde_json::{json, Map, Value};\n", "use serde_json::{json, Value};\n", 1)
legacy = legacy.replace("use sha2::{Digest, Sha256};\n", "", 1)
legacy = legacy.replace("use std::collections::{BTreeMap, BTreeSet};\n", "", 1)
legacy = legacy.replace(
    "            P4_FREEZE_JOB => {\n                self.execute_p4_freeze_task(store, payload.task_id, job_id)\n                    .await\n            }\n",
    "            P4_FREEZE_JOB => self.execute_p4_freeze_task(payload.task_id, job_id).await,\n",
    1,
)
freeze_start = legacy.find("    async fn execute_p4_freeze_task(\n")
freeze_end = legacy.find("\n}\n\npub(crate) fn spawn_p4_orchestration_worker", freeze_start)
if freeze_start == -1 or freeze_end == -1:
    raise RuntimeError("legacy freeze method block not found")
legacy = legacy[:freeze_start] + legacy[freeze_end:]
helper_start = legacy.find("fn attach_orchestration_input(\n")
helper_end = legacy.find("#[cfg(test)]\nmod tests", helper_start)
if helper_start == -1 or helper_end == -1:
    raise RuntimeError("legacy freeze helper block not found")
legacy = legacy[:helper_start] + legacy[helper_end:]
legacy = legacy.replace(
    "mod tests {\n    use super::*;\n    use crate::use_cases::prediction::shared::p4_planning::is_p4_model;\n    use football_domain::P4Horizon;\n",
    "mod tests {\n    use super::*;\n    use crate::use_cases::prediction::shared::p4_planning::{canonical_fact_keys, is_p4_model};\n    use football_domain::P4Horizon;\n    use std::collections::BTreeSet;\n",
    1,
)
write(legacy_path, legacy)

# 8. Persistent specialty verifier for this responsibility boundary.
write(
    "scripts/verify-r3-06-p4-freeze.mjs",
    r'''import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const read = (file) => fs.readFileSync(path.join(root, file), "utf8");
const failures = [];
const check = (condition, message) => { if (!condition) failures.push(message); };

const port = read("crates/application/src/ports/prediction/mod.rs");
const adapter = read("crates/application/src/composition/adapters/prediction.rs");
const useCase = read("crates/application/src/use_cases/prediction/execute_p4_freeze/mod.rs");
const projection = read("crates/application/src/use_cases/prediction/execute_p4_freeze/snapshot_projection.rs");
const service = read("crates/application/src/services/prediction/service.rs");
const facade = read("crates/application/src/services/prediction/facade.rs");
const legacy = read("crates/application/src/p4_orchestration.rs");

for (const token of ["P4FreezeExecutionPort", "find_frozen_snapshot_id", "routed_facts", "freeze_snapshot"]) {
  check(port.includes(token), `Prediction freeze Port 缺少：${token}`);
  check(adapter.includes(token), `Prediction freeze adapter 缺少：${token}`);
}
for (const token of ["execute_prediction::execute", "validate_pinned_route", "snapshot_probabilities", "snapshot_features", "freeze_snapshot", "P4FreezeTaskState::Frozen"]) {
  check(useCase.includes(token) || projection.includes(token), `P4 freeze use case 缺少：${token}`);
}
check(!useCase.includes("PersistenceStore") && !projection.includes("PersistenceStore"), "P4 freeze use case 不得依赖具体 PersistenceStore");
check(!useCase.includes("football_persistence_postgres") && !projection.includes("football_persistence_postgres"), "P4 freeze use case 不得导入 PostgreSQL crate");
check(!useCase.includes("create_research_run") && !useCase.includes("append_evidence_claim") && !useCase.includes("execute_p4_openai_research"), "P4 freeze use case 不得接管 Research 写入/联网执行职责");
check(service.includes("execute_p4_freeze::execute") && facade.includes("execute_p4_freeze_task"), "Prediction Service/facade 未接通 freeze use case");
check(!legacy.includes("async fn execute_p4_freeze_task("), "旧 p4_orchestration 仍保留 freeze 实现");
check(legacy.includes("P4_FREEZE_JOB => self.execute_p4_freeze_task(payload.task_id, job_id).await"), "旧混合 worker 未委托 Prediction freeze service");
check(legacy.includes("execute_p4_research_task") && legacy.includes("execute_p4_openai_research"), "R3-07 Research 执行边界被意外迁出");

if (failures.length) {
  console.error("R3-06 P4 Freeze Execution 验证失败：\n- " + failures.join("\n- "));
  process.exit(1);
}
console.log("R3-06 P4 Freeze Execution 验证通过：冻结状态机、固定路由、31字段/概率投影与不可变快照已由 Prediction Service 经 Ports 接管，Research 写入链保持隔离。");
''',
)

print("R3-06 Atomic Task 2B migration generated")
