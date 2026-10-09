use crate::use_cases::prediction::shared::p4_planning::horizon_priority;
use crate::use_cases::prediction::P4PlanningAccess;
use crate::ApplicationResult;
use chrono::{DateTime, Utc};
use football_domain::{
    EnqueueJobDraft, P4FreezeTaskRecord, P4FreezeTaskState, P4FreezeTaskTransition,
};
use serde_json::{json, Value};

const P4_RESEARCH_JOB: &str = "p4_horizon_research";

pub(super) async fn resume<P: P4PlanningAccess + ?Sized>(
    port: &P,
    task: &P4FreezeTaskRecord,
    now: DateTime<Utc>,
) -> ApplicationResult<P4FreezeTaskRecord> {
    if task.state != P4FreezeTaskState::Planned {
        return Ok(task.clone());
    }
    if task.data_cutoff_at <= now {
        return Ok(port
            .transition_freeze_task(
                task.id,
                &P4FreezeTaskTransition {
                    task_id: task.id,
                    expected_state: P4FreezeTaskState::Planned,
                    next_state: P4FreezeTaskState::Missed,
                    reason: "重试规划时正式截止时点已过，未进入研究队列".to_string(),
                    blockers: Value::Null,
                    payload: json!({"planner_retry": true}),
                    research_run_id: None,
                    research_job_id: None,
                    freeze_job_id: None,
                    snapshot_id: None,
                },
            )
            .await?);
    }
    let research_job = port
        .enqueue(&EnqueueJobDraft {
            job_type: P4_RESEARCH_JOB.to_string(),
            payload: json!({"task_id": task.id}),
            idempotency_key: Some(format!("p4-research-job:{}", task.id)),
            available_at: Some(task.research_due_at),
            priority: horizon_priority(task.horizon),
            max_attempts: 3,
        })
        .await?;
    let queued = port
        .transition_freeze_task(
            task.id,
            &P4FreezeTaskTransition {
                task_id: task.id,
                expected_state: P4FreezeTaskState::Planned,
                next_state: P4FreezeTaskState::ResearchQueued,
                reason: "研究任务已进入预约后台队列".to_string(),
                blockers: Value::Null,
                payload: json!({
                    "job_id": research_job.id,
                    "available_at": research_job.available_at,
                }),
                research_run_id: None,
                research_job_id: Some(research_job.id),
                freeze_job_id: None,
                snapshot_id: None,
            },
        )
        .await?;
    Ok(queued)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::p4_orchestration::tests::task_fixture;
    use crate::use_cases::prediction::tests::Probe;
    use chrono::Duration;

    #[tokio::test]
    async fn planned_resume_uses_inclusive_cutoff_and_preserves_other_states() {
        let cutoff = Utc::now();
        for (state, now, queued) in [
            (
                P4FreezeTaskState::Planned,
                cutoff - Duration::nanoseconds(1),
                true,
            ),
            (P4FreezeTaskState::Planned, cutoff, false),
            (
                P4FreezeTaskState::Planned,
                cutoff + Duration::nanoseconds(1),
                false,
            ),
            (P4FreezeTaskState::ResearchQueued, cutoff, false),
        ] {
            let probe = Probe::new();
            let mut task = task_fixture(state);
            task.data_cutoff_at = cutoff;
            probe.state.lock().unwrap().task = Some(task.clone());
            let result = resume(&probe, &task, now).await.unwrap();
            let value = probe.state.lock().unwrap();
            if state != P4FreezeTaskState::Planned {
                assert_eq!(
                    serde_json::to_value(result).unwrap(),
                    serde_json::to_value(task).unwrap()
                );
                assert!(value.calls.is_empty());
                continue;
            }
            assert_eq!(value.enqueues.len(), usize::from(queued));
            assert_eq!(value.transitions.len(), 1);
            assert_eq!(
                result.state,
                if queued {
                    P4FreezeTaskState::ResearchQueued
                } else {
                    P4FreezeTaskState::Missed
                }
            );
            if !queued {
                assert_eq!(value.transitions[0].payload, json!({"planner_retry":true}));
                assert_eq!(result.research_job_id, None);
            }
        }
    }
}
