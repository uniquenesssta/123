use super::{context::research_dynamic_context, transitions::*};
use crate::ports::{
    analytics::JobQueuePort,
    prediction::PredictionWorkflowPort,
    research::{ResearchArtifactPort, ResearchGatewayAuditPort},
};
use crate::use_cases::research::{
    fact_pipeline::FactPipelineAccess,
    openai_gateway::{self, OpenAiResearchCommand},
};
use crate::{ApplicationError, ApplicationResult};
use chrono::Utc;
use football_domain::{
    P4FreezeTaskState, P4FreezeTaskTransition, ResearchRunDraft, ResearchRunStatus,
    P4_ORCHESTRATION_PLANNER_VERSION,
};
use football_research_gateway::{CancellationToken, GatewayOperation};
use serde_json::{json, Value};
use uuid::Uuid;

pub(super) async fn execute(
    workflow: &dyn PredictionWorkflowPort,
    jobs: &dyn JobQueuePort,
    artifacts: &dyn ResearchArtifactPort,
    audit: &dyn ResearchGatewayAuditPort,
    pipeline: &dyn FactPipelineAccess,
    task_id: Uuid,
    job_id: Uuid,
) -> ApplicationResult<Value> {
    let mut task = workflow.read_freeze_task(task_id).await?;
    if task.state.is_terminal() {
        return Ok(json!({"task_id": task.id, "state": task.state, "noop": true}));
    }
    if matches!(
        task.state,
        P4FreezeTaskState::ReadyToFreeze | P4FreezeTaskState::Freezing
    ) {
        return Ok(json!({"task_id": task.id, "state": task.state, "noop": true}));
    }
    if task.state == P4FreezeTaskState::ResearchSucceeded {
        let ready = finalize_successful_research(workflow, jobs, &task).await?;
        return Ok(json!({"task_id": ready.id, "state": ready.state}));
    }
    if task.state == P4FreezeTaskState::ResearchPartial {
        let blocked = block_partial_research(workflow, &task).await?;
        return Ok(json!({"task_id": blocked.id, "state": blocked.state}));
    }
    let now = Utc::now();
    if now >= task.data_cutoff_at {
        task = transition_missed(
            workflow,
            &task,
            "研究任务未能在数据截止时间前开始",
            json!({"job_id": job_id, "now": now}),
        )
        .await?;
        return Ok(json!({"task_id": task.id, "state": task.state}));
    }
    if task.state == P4FreezeTaskState::Planned {
        task = workflow
            .transition_freeze_task(
                task.id,
                &P4FreezeTaskTransition {
                    task_id: task.id,
                    expected_state: P4FreezeTaskState::Planned,
                    next_state: P4FreezeTaskState::ResearchQueued,
                    reason: "恢复已创建但尚未登记队列状态的研究任务".to_string(),
                    blockers: Value::Null,
                    payload: json!({"job_id": job_id, "recovered": true}),
                    research_run_id: None,
                    research_job_id: Some(job_id),
                    freeze_job_id: None,
                    snapshot_id: None,
                },
            )
            .await?;
    }
    if task.state == P4FreezeTaskState::ResearchQueued {
        let research_run = artifacts
            .create_run(&ResearchRunDraft {
                match_id: task.match_id,
                horizon: task.horizon,
                data_cutoff_at: task.data_cutoff_at,
                trace_id: task.trace_id,
                idempotency_key: format!("p4-research-run:{}", task.id),
                planner_version: Some(P4_ORCHESTRATION_PLANNER_VERSION.to_string()),
                prompt_version_id: None,
                schema_version_id: task.research_schema_version_id,
                request_payload: json!({
                    "requested_fact_keys": task.requested_fact_keys.clone(),
                    "orchestration_task_id": task.id,
                }),
                metadata: json!({
                    "stage": "F",
                    "job_id": job_id,
                }),
            })
            .await?;
        task = workflow
            .transition_freeze_task(
                task.id,
                &P4FreezeTaskTransition {
                    task_id: task.id,
                    expected_state: P4FreezeTaskState::ResearchQueued,
                    next_state: P4FreezeTaskState::ResearchRunning,
                    reason: "联网事实研究开始".to_string(),
                    blockers: Value::Null,
                    payload: json!({"job_id": job_id, "research_run_id": research_run.id}),
                    research_run_id: Some(research_run.id),
                    research_job_id: None,
                    freeze_job_id: None,
                    snapshot_id: None,
                },
            )
            .await?;
    }
    if task.state != P4FreezeTaskState::ResearchRunning {
        return Err(ApplicationError::Validation(format!(
            "研究任务状态不是RESEARCH_RUNNING：{}",
            task.state.as_str()
        )));
    }
    let research_run_id = task.research_run_id.ok_or_else(|| {
        ApplicationError::Validation("RESEARCH_RUNNING任务缺少research_run_id".to_string())
    })?;
    let existing_run = artifacts.read_run(research_run_id).await?;
    match existing_run.status {
        ResearchRunStatus::Cancelled => {
            let cancelled = workflow
                .transition_freeze_task(
                    task.id,
                    &P4FreezeTaskTransition {
                        task_id: task.id,
                        expected_state: P4FreezeTaskState::ResearchRunning,
                        next_state: P4FreezeTaskState::Cancelled,
                        reason: "恢复任务时发现研究任务已取消".to_string(),
                        blockers: json!(["research_run_cancelled"]),
                        payload: json!({"research_run_id": research_run_id, "job_id": job_id}),
                        research_run_id: None,
                        research_job_id: None,
                        freeze_job_id: None,
                        snapshot_id: None,
                    },
                )
                .await?;
            return Ok(json!({"task_id": cancelled.id, "state": cancelled.state}));
        }
        ResearchRunStatus::Succeeded | ResearchRunStatus::Partial => {}
        ResearchRunStatus::Planned | ResearchRunStatus::Running | ResearchRunStatus::Failed => {
            let dynamic_context = research_dynamic_context(workflow, &task).await?;
            if let Err(error) = openai_gateway::execute_p4_openai_research(
                artifacts,
                audit,
                pipeline,
                OpenAiResearchCommand {
                    research_run_id,
                    trace_id: task.trace_id,
                    match_key: task.match_key.clone(),
                    data_cutoff_at: task.data_cutoff_at,
                    operation: GatewayOperation::Research,
                    dynamic_context,
                    requested_fact_keys: task.requested_fact_keys.clone(),
                },
                CancellationToken::new(),
            )
            .await
            {
                let failed_run = artifacts.read_run(research_run_id).await?;
                if failed_run.status == ResearchRunStatus::Partial {
                    let partial = workflow
                        .transition_freeze_task(
                            task.id,
                            &P4FreezeTaskTransition {
                                task_id: task.id,
                                expected_state: P4FreezeTaskState::ResearchRunning,
                                next_state: P4FreezeTaskState::ResearchPartial,
                                reason: "联网结果已保存，但事实流水线存在阻断".to_string(),
                                blockers: json!([error.to_string()]),
                                payload: json!({"job_id": job_id, "research_run_id": research_run_id}),
                                research_run_id: None,
                                research_job_id: None,
                                freeze_job_id: None,
                                snapshot_id: None,
                            },
                        )
                        .await?;
                    let blocked = block_partial_research(workflow, &partial).await?;
                    return Ok(json!({"task_id": blocked.id, "state": blocked.state}));
                }
                return Err(error);
            }
        }
    }
    let run = artifacts.read_run(research_run_id).await?;
    match run.status {
        ResearchRunStatus::Succeeded => {
            task = workflow
                .transition_freeze_task(
                    task.id,
                    &P4FreezeTaskTransition {
                        task_id: task.id,
                        expected_state: P4FreezeTaskState::ResearchRunning,
                        next_state: P4FreezeTaskState::ResearchSucceeded,
                        reason: "联网事实研究与证据路由成功".to_string(),
                        blockers: Value::Null,
                        payload: json!({"research_run_id": research_run_id}),
                        research_run_id: None,
                        research_job_id: None,
                        freeze_job_id: None,
                        snapshot_id: None,
                    },
                )
                .await?;
            let ready = finalize_successful_research(workflow, jobs, &task).await?;
            Ok(json!({"task_id": ready.id, "state": ready.state}))
        }
        ResearchRunStatus::Partial => {
            task = workflow
                .transition_freeze_task(
                    task.id,
                    &P4FreezeTaskTransition {
                        task_id: task.id,
                        expected_state: P4FreezeTaskState::ResearchRunning,
                        next_state: P4FreezeTaskState::ResearchPartial,
                        reason: "联网事实研究存在实体、时间、来源或冲突阻断".to_string(),
                        blockers: json!(["research_run_partial"]),
                        payload: json!({"research_run_id": research_run_id}),
                        research_run_id: None,
                        research_job_id: None,
                        freeze_job_id: None,
                        snapshot_id: None,
                    },
                )
                .await?;
            let blocked = block_partial_research(workflow, &task).await?;
            Ok(json!({"task_id": blocked.id, "state": blocked.state}))
        }
        status => Err(ApplicationError::Validation(format!(
            "研究任务未进入终态：{}",
            status.as_str()
        ))),
    }
}
