use crate::ports::{
    analytics::JobQueuePort,
    prediction::P4OrchestrationQueuePort,
    research::{ResearchArtifactPort, ResearchGatewayAuditPort},
};
use crate::use_cases::{
    prediction::P4FreezeExecutionAccess, research::fact_pipeline::FactPipelineAccess,
};

use super::{dispatch, failure};
use crate::model_registry::ModelRegistry;
use crate::ports::prediction::SerializedP4OrchestrationResult;
use crate::services::{prediction::PredictionService, research::ResearchService};
use crate::ApplicationResult;
use serde_json::Value;

pub(crate) async fn execute<P: P4OrchestrationAccess>(
    port: &P,
    registry: &ModelRegistry,
    research: &ResearchService,
    prediction: &PredictionService,
) -> ApplicationResult<Option<Value>> {
    let Some(job) = P4OrchestrationQueuePort::claim_next_p4_job(
        port,
        &[dispatch::P4_RESEARCH_JOB, dispatch::P4_FREEZE_JOB],
    )
    .await?
    else {
        return Ok(None);
    };

    let result = dispatch::execute_job(
        port,
        registry,
        research,
        prediction,
        &job.job_type,
        &job.payload,
        job.id,
    )
    .await;
    settle_job(port, &job, result).await
}

pub(super) async fn settle_job<
    P: crate::ports::prediction::PredictionWorkflowPort + P4OrchestrationQueuePort + ?Sized,
>(
    port: &P,
    job: &football_domain::BackgroundJob,
    result: ApplicationResult<Value>,
) -> ApplicationResult<Option<Value>> {
    match result {
        Ok(value) => {
            let serialized = SerializedP4OrchestrationResult::new(serde_json::to_string(&value)?);
            P4OrchestrationQueuePort::complete_p4_job(port, job.id, &serialized).await?;
            Ok(Some(value))
        }
        Err(error) => {
            failure::mark_terminal_failure(port, job, &error.to_string()).await;
            P4OrchestrationQueuePort::fail_p4_job(port, job.id, &error.to_string()).await?;
            Err(error)
        }
    }
}

pub(crate) trait P4OrchestrationAccess:
    P4OrchestrationQueuePort
    + P4FreezeExecutionAccess
    + JobQueuePort
    + ResearchArtifactPort
    + ResearchGatewayAuditPort
    + FactPipelineAccess
{
}

impl<T> P4OrchestrationAccess for T where
    T: P4OrchestrationQueuePort
        + P4FreezeExecutionAccess
        + JobQueuePort
        + ResearchArtifactPort
        + ResearchGatewayAuditPort
        + FactPipelineAccess
        + ?Sized
{
}
