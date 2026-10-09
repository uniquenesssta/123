use super::P4OrchestrationAccess;
use crate::model_registry::ModelRegistry;
use crate::services::{prediction::PredictionService, research::ResearchService};
use crate::use_cases::research::p4_worker::P4ResearchWorkerAccess;
use crate::{ApplicationError, ApplicationResult};
use serde::Deserialize;
use serde_json::Value;
use uuid::Uuid;

pub(super) const P4_RESEARCH_JOB: &str = "p4_horizon_research";
pub(super) const P4_FREEZE_JOB: &str = "p4_horizon_freeze";

#[derive(Debug, Deserialize)]
pub(super) struct OrchestrationJobPayload {
    pub(super) task_id: Uuid,
}

pub(super) async fn execute_job<P: P4OrchestrationAccess>(
    port: &P,
    registry: &ModelRegistry,
    research: &ResearchService,
    prediction: &PredictionService,
    job_type: &str,
    payload: &Value,
    job_id: Uuid,
) -> ApplicationResult<Value> {
    let payload: OrchestrationJobPayload = serde_json::from_value(payload.clone())?;
    match job_type {
        P4_RESEARCH_JOB => {
            research
                .execute_p4_research_task(
                    P4ResearchWorkerAccess {
                        workflow: port,
                        jobs: port,
                        artifacts: port,
                        audit: port,
                        pipeline: port,
                    },
                    payload.task_id,
                    job_id,
                )
                .await
        }
        P4_FREEZE_JOB => {
            prediction
                .execute_p4_freeze_task(port, registry, payload.task_id, job_id)
                .await
        }
        other => Err(ApplicationError::Validation(format!(
            "P4编排器不支持后台任务：{other}"
        ))),
    }
}
