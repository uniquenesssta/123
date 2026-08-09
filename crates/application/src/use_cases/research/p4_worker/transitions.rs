use crate::ports::{analytics::JobQueuePort, prediction::PredictionWorkflowPort};
use crate::use_cases::prediction::shared::p4_planning::horizon_priority;
use crate::{ApplicationError, ApplicationResult};
use football_domain::{
    EnqueueJobDraft, P4FreezeTaskRecord, P4FreezeTaskState, P4FreezeTaskTransition,
};
use serde_json::{json, Value};

const P4_FREEZE_JOB: &str = "p4_horizon_freeze";

pub(super) async fn finalize_successful_research(
    workflow: &dyn PredictionWorkflowPort,
    jobs: &dyn JobQueuePort,
    task: &P4FreezeTaskRecord,
) -> ApplicationResult<P4FreezeTaskRecord> {
    let readiness = workflow.freeze_readiness(task.id).await?;
    if !readiness.ready {
        return Ok(workflow
            .transition_freeze_task(
                task.id,
                &P4FreezeTaskTransition {
                    task_id: task.id,
                    expected_state: P4FreezeTaskState::ResearchSucceeded,
                    next_state: P4FreezeTaskState::Blocked,
                    reason: "研究完成但未通过READY_TO_FREEZE门禁".to_string(),
                    blockers: serde_json::to_value(&readiness.blockers)?,
                    payload: serde_json::to_value(&readiness)?,
                    research_run_id: None,
                    research_job_id: None,
                    freeze_job_id: None,
                    snapshot_id: None,
                },
            )
            .await?);
    }
    let freeze_job = jobs
        .enqueue(&EnqueueJobDraft {
            job_type: P4_FREEZE_JOB.to_string(),
            payload: json!({"task_id": task.id}),
            idempotency_key: Some(format!("p4-freeze-job:{}", task.id)),
            available_at: Some(task.data_cutoff_at),
            priority: horizon_priority(task.horizon) + 100,
            max_attempts: 3,
        })
        .await?;
    Ok(workflow
        .transition_freeze_task(
            task.id,
            &P4FreezeTaskTransition {
                task_id: task.id,
                expected_state: P4FreezeTaskState::ResearchSucceeded,
                next_state: P4FreezeTaskState::ReadyToFreeze,
                reason: "所有事实路由通过门禁，等待截止时点自动冻结".to_string(),
                blockers: json!([]),
                payload: json!({
                    "readiness": readiness,
                    "freeze_job_id": freeze_job.id,
                    "available_at": freeze_job.available_at,
                }),
                research_run_id: None,
                research_job_id: None,
                freeze_job_id: Some(freeze_job.id),
                snapshot_id: None,
            },
        )
        .await?)
}

pub(super) async fn block_partial_research(
    workflow: &dyn PredictionWorkflowPort,
    task: &P4FreezeTaskRecord,
) -> ApplicationResult<P4FreezeTaskRecord> {
    let readiness = workflow.freeze_readiness(task.id).await?;
    Ok(workflow
        .transition_freeze_task(
            task.id,
            &P4FreezeTaskTransition {
                task_id: task.id,
                expected_state: P4FreezeTaskState::ResearchPartial,
                next_state: P4FreezeTaskState::Blocked,
                reason: "部分研究结果不得进入正式冻结队列".to_string(),
                blockers: serde_json::to_value(&readiness.blockers)?,
                payload: serde_json::to_value(&readiness)?,
                research_run_id: None,
                research_job_id: None,
                freeze_job_id: None,
                snapshot_id: None,
            },
        )
        .await?)
}

pub(super) async fn transition_missed(
    workflow: &dyn PredictionWorkflowPort,
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
    Ok(workflow
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
