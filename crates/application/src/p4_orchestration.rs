use super::{ApplicationError, ApplicationResult, ApplicationService};
use football_domain::{P4FreezeTaskState, P4FreezeTaskTransition};
use serde::Deserialize;
use serde_json::{json, Value};
use std::sync::{atomic::Ordering, Arc};
use tokio::time::{sleep, Duration as TokioDuration};
use uuid::Uuid;

const P4_RESEARCH_JOB: &str = "p4_horizon_research";
const P4_FREEZE_JOB: &str = "p4_horizon_freeze";
const P4_WORKER_POLL_SECONDS: u64 = 30;

#[derive(Debug, Deserialize)]
struct OrchestrationJobPayload {
    task_id: Uuid,
}

impl ApplicationService {
    pub async fn process_next_p4_orchestration_job(
        self: &Arc<Self>,
    ) -> ApplicationResult<Option<Value>> {
        let store = self.active_store().await?;
        let Some(job) = store
            .claim_next_job_by_types(&[P4_RESEARCH_JOB, P4_FREEZE_JOB])
            .await?
        else {
            return Ok(None);
        };
        let result = self
            .execute_p4_orchestration_job(&job.job_type, &job.payload, job.id)
            .await;
        match result {
            Ok(value) => {
                store.complete_job(job.id, value.clone()).await?;
                Ok(Some(value))
            }
            Err(error) => {
                if job.attempts >= job.max_attempts {
                    if let Ok(payload) =
                        serde_json::from_value::<OrchestrationJobPayload>(job.payload.clone())
                    {
                        if let Ok(task) = store.read_p4_freeze_task(payload.task_id).await {
                            if !task.state.is_terminal()
                                && task.state.can_transition_to(P4FreezeTaskState::Failed)
                            {
                                let _ = store
                                    .transition_p4_freeze_task(&P4FreezeTaskTransition {
                                        task_id: task.id,
                                        expected_state: task.state,
                                        next_state: P4FreezeTaskState::Failed,
                                        reason: "P4编排任务达到最大尝试次数".to_string(),
                                        blockers: json!([error.to_string()]),
                                        payload: json!({
                                            "job_id": job.id,
                                            "attempts": job.attempts,
                                            "max_attempts": job.max_attempts,
                                        }),
                                        research_run_id: None,
                                        research_job_id: None,
                                        freeze_job_id: None,
                                        snapshot_id: None,
                                    })
                                    .await;
                            }
                        }
                    }
                }
                store.fail_job(job.id, &error.to_string()).await?;
                Err(error)
            }
        }
    }

    async fn execute_p4_orchestration_job(
        &self,
        job_type: &str,
        payload: &Value,
        job_id: Uuid,
    ) -> ApplicationResult<Value> {
        let payload: OrchestrationJobPayload = serde_json::from_value(payload.clone())?;
        match job_type {
            P4_RESEARCH_JOB => self.execute_p4_research_task(payload.task_id, job_id).await,
            P4_FREEZE_JOB => self.execute_p4_freeze_task(payload.task_id, job_id).await,
            other => Err(ApplicationError::Validation(format!(
                "P4编排器不支持后台任务：{other}"
            ))),
        }
    }
}

pub(crate) fn spawn_p4_orchestration_worker(service: Arc<ApplicationService>) {
    if service.p4_worker_running.swap(true, Ordering::SeqCst) {
        return;
    }
    tokio::spawn(async move {
        loop {
            match service.process_next_p4_orchestration_job().await {
                Ok(Some(_)) => continue,
                Ok(None) => sleep(TokioDuration::from_secs(P4_WORKER_POLL_SECONDS)).await,
                Err(ApplicationError::DatabaseNotConnected) => break,
                Err(_) => sleep(TokioDuration::from_secs(P4_WORKER_POLL_SECONDS)).await,
            }
        }
        service.p4_worker_running.store(false, Ordering::SeqCst);
    });
}

#[cfg(test)]
mod tests {
    use crate::use_cases::prediction::shared::p4_planning::{
        canonical_fact_keys, horizon_priority, is_p4_model,
    };
    use football_domain::P4Horizon;
    use std::collections::BTreeSet;

    #[test]
    fn formal_fact_set_has_twenty_nine_unique_fields() {
        let fields = canonical_fact_keys();
        assert_eq!(fields.len(), 29);
        assert_eq!(fields.iter().collect::<BTreeSet<_>>().len(), 29);
    }

    #[test]
    fn only_p4_models_enter_stage_f() {
        assert!(is_p4_model("p4"));
        assert!(is_p4_model("p4_knockout_90"));
        assert!(!is_p4_model("p7"));
    }

    #[test]
    fn canonical_horizon_priority_increases_toward_kickoff() {
        assert!(horizon_priority(P4Horizon::T24h) < horizon_priority(P4Horizon::T6h));
        assert!(horizon_priority(P4Horizon::T6h) < horizon_priority(P4Horizon::T90m));
        assert!(horizon_priority(P4Horizon::T90m) < horizon_priority(P4Horizon::T1h));
    }
}
