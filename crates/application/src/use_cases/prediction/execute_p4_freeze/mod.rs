mod snapshot_projection;

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
