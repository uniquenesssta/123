use super::P4ManualConflictAccess;
use crate::use_cases::research::p4_worker;
use crate::{ApplicationError, ApplicationResult};
use football_domain::{
    P4FreezeTaskState, P4FreezeTaskTransition, ResearchRunEventDraft, ResearchRunStatus,
};
use serde_json::json;
use uuid::Uuid;

pub(super) async fn reconcile(
    access: &P4ManualConflictAccess<'_>,
    task_id: Uuid,
    research_run_id: Uuid,
) -> ApplicationResult<()> {
    let route_readiness = access.manual.route_readiness(task_id).await?;
    if route_readiness.ready {
        access
            .artifacts
            .record_run_event(&ResearchRunEventDraft {
                research_run_id,
                idempotency_key: format!("manual-review-succeeded:{task_id}"),
                status: ResearchRunStatus::Succeeded,
                response_id: None,
                model_id: None,
                token_usage: json!({}),
                error_category: None,
                error_message: None,
                payload: json!({
                    "stage": "G",
                    "task_id": task_id,
                    "reason": "all immutable routes passed after append-only manual conflict decisions"
                }),
            })
            .await?;

        for _ in 0..4 {
            let current = access.workflow.read_freeze_task(task_id).await?;
            let recovered = match current.state {
                P4FreezeTaskState::ResearchPartial | P4FreezeTaskState::Blocked => match access
                    .workflow
                    .transition_freeze_task(
                        task_id,
                        &P4FreezeTaskTransition {
                            task_id,
                            expected_state: current.state,
                            next_state: P4FreezeTaskState::ResearchSucceeded,
                            reason: "人工冲突处理完成，全部事实路由重新通过门禁".to_string(),
                            blockers: json!([]),
                            payload: serde_json::to_value(&route_readiness)?,
                            research_run_id: None,
                            research_job_id: None,
                            freeze_job_id: None,
                            snapshot_id: None,
                        },
                    )
                    .await
                {
                    Ok(task) => task,
                    Err(error) => {
                        let latest = access.workflow.read_freeze_task(task_id).await?;
                        if latest.state != current.state {
                            continue;
                        }
                        return Err(error.into());
                    }
                },
                P4FreezeTaskState::ResearchSucceeded => current,
                P4FreezeTaskState::ReadyToFreeze
                | P4FreezeTaskState::Freezing
                | P4FreezeTaskState::Frozen
                | P4FreezeTaskState::Missed
                | P4FreezeTaskState::Failed
                | P4FreezeTaskState::Cancelled => return Ok(()),
                other => {
                    return Err(ApplicationError::Validation(format!(
                        "人工冲突决策完成后无法从状态{}恢复冻结链",
                        other.as_str()
                    )))
                }
            };

            match p4_worker::finalize_successful_research(access.workflow, access.jobs, &recovered)
                .await
            {
                Ok(_) => return Ok(()),
                Err(error) => {
                    let latest = access.workflow.read_freeze_task(task_id).await?;
                    if matches!(
                        latest.state,
                        P4FreezeTaskState::ReadyToFreeze
                            | P4FreezeTaskState::Freezing
                            | P4FreezeTaskState::Frozen
                    ) {
                        return Ok(());
                    }
                    if latest.state == P4FreezeTaskState::ResearchSucceeded {
                        continue;
                    }
                    return Err(error);
                }
            }
        }
        return Err(ApplicationError::Validation(
            "人工冲突处理后的任务状态持续发生并发变化，请刷新工作台确认最终状态".to_string(),
        ));
    }

    let current = access.workflow.read_freeze_task(task_id).await?;
    match current.state {
        P4FreezeTaskState::ResearchPartial => {
            match access
                .workflow
                .transition_freeze_task(
                    task_id,
                    &P4FreezeTaskTransition {
                        task_id,
                        expected_state: P4FreezeTaskState::ResearchPartial,
                        next_state: P4FreezeTaskState::Blocked,
                        reason: "人工处理后仍存在其他阻断项".to_string(),
                        blockers: serde_json::to_value(&route_readiness.blockers)?,
                        payload: serde_json::to_value(&route_readiness)?,
                        research_run_id: None,
                        research_job_id: None,
                        freeze_job_id: None,
                        snapshot_id: None,
                    },
                )
                .await
            {
                Ok(_) => Ok(()),
                Err(error) => {
                    let latest = access.workflow.read_freeze_task(task_id).await?;
                    if latest.state != P4FreezeTaskState::ResearchPartial {
                        Ok(())
                    } else {
                        Err(error.into())
                    }
                }
            }
        }
        P4FreezeTaskState::Blocked
        | P4FreezeTaskState::ResearchSucceeded
        | P4FreezeTaskState::ReadyToFreeze
        | P4FreezeTaskState::Freezing
        | P4FreezeTaskState::Frozen
        | P4FreezeTaskState::Missed
        | P4FreezeTaskState::Failed
        | P4FreezeTaskState::Cancelled => Ok(()),
        other => Err(ApplicationError::Validation(format!(
            "人工处理后无法从状态{}登记剩余阻断项",
            other.as_str()
        ))),
    }
}
