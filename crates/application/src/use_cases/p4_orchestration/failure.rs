use super::P4OrchestrationAccess;
use crate::ports::prediction::PredictionWorkflowPort;
use football_domain::{BackgroundJob, P4FreezeTaskState, P4FreezeTaskTransition};
use serde::Deserialize;
use serde_json::json;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub(super) struct OrchestrationJobPayload {
    pub(super) task_id: Uuid,
}

pub(super) async fn mark_terminal_failure<P: P4OrchestrationAccess + ?Sized>(
    port: &P,
    job: &BackgroundJob,
    error_message: &str,
) {
    if job.attempts < job.max_attempts {
        return;
    }
    let Ok(payload) = serde_json::from_value::<OrchestrationJobPayload>(job.payload.clone()) else {
        return;
    };
    let Ok(task) = PredictionWorkflowPort::read_freeze_task(port, payload.task_id).await else {
        return;
    };
    if task.state.is_terminal() || !task.state.can_transition_to(P4FreezeTaskState::Failed) {
        return;
    }
    let _ = PredictionWorkflowPort::transition_freeze_task(
        port,
        task.id,
        &P4FreezeTaskTransition {
            task_id: task.id,
            expected_state: task.state,
            next_state: P4FreezeTaskState::Failed,
            reason: "P4编排任务达到最大尝试次数".to_string(),
            blockers: json!([error_message]),
            payload: json!({
                "job_id": job.id,
                "attempts": job.attempts,
                "max_attempts": job.max_attempts,
            }),
            research_run_id: None,
            research_job_id: None,
            freeze_job_id: None,
            snapshot_id: None,
        },
    )
    .await;
}
