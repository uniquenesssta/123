use super::{failure, P4OrchestrationAccess};
use crate::model_registry::ModelRegistry;
use crate::ports::prediction::{P4OrchestrationQueuePort, SerializedP4OrchestrationResult};
use crate::services::{prediction::PredictionService, research::ResearchService};
use crate::use_cases::research::p4_worker::P4ResearchWorkerAccess;
use crate::{ApplicationError, ApplicationResult};
use serde_json::Value;
use uuid::Uuid;

const P4_RESEARCH_JOB: &str = "p4_horizon_research";
const P4_FREEZE_JOB: &str = "p4_horizon_freeze";

pub(crate) async fn execute<P: P4OrchestrationAccess>(
    port: &P,
    registry: &ModelRegistry,
    research: &ResearchService,
    prediction: &PredictionService,
) -> ApplicationResult<Option<Value>> {
    let Some(job) =
        P4OrchestrationQueuePort::claim_next_p4_job(port, &[P4_RESEARCH_JOB, P4_FREEZE_JOB])
            .await?
    else {
        return Ok(None);
    };

    let result = execute_job(
        port,
        registry,
        research,
        prediction,
        &job.job_type,
        &job.payload,
        job.id,
    )
    .await;
    match result {
        Ok(value) => {
            let serialized = SerializedP4OrchestrationResult::new(serde_json::to_string(&value)?);
            P4OrchestrationQueuePort::complete_p4_job(port, job.id, &serialized).await?;
            Ok(Some(value))
        }
        Err(error) => {
            failure::mark_terminal_failure(port, &job, &error.to_string()).await;
            P4OrchestrationQueuePort::fail_p4_job(port, job.id, &error.to_string()).await?;
            Err(error)
        }
    }
}

async fn execute_job<P: P4OrchestrationAccess>(
    port: &P,
    registry: &ModelRegistry,
    research: &ResearchService,
    prediction: &PredictionService,
    job_type: &str,
    payload: &Value,
    job_id: Uuid,
) -> ApplicationResult<Value> {
    let payload: failure::OrchestrationJobPayload = serde_json::from_value(payload.clone())?;
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
