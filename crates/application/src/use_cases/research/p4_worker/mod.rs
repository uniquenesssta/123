use crate::ports::{
    analytics::JobQueuePort,
    prediction::PredictionWorkflowPort,
    research::{ResearchArtifactPort, ResearchGatewayAuditPort},
};
use crate::use_cases::research::fact_pipeline::FactPipelineAccess;
use crate::ApplicationResult;
use football_domain::P4FreezeTaskRecord;
use serde_json::Value;
use uuid::Uuid;

mod context;
mod execution;
mod transitions;

pub(crate) struct P4ResearchWorkerAccess<'a> {
    pub(crate) workflow: &'a dyn PredictionWorkflowPort,
    pub(crate) jobs: &'a dyn JobQueuePort,
    pub(crate) artifacts: &'a dyn ResearchArtifactPort,
    pub(crate) audit: &'a dyn ResearchGatewayAuditPort,
    pub(crate) pipeline: &'a dyn FactPipelineAccess,
}

pub(crate) async fn execute(
    access: P4ResearchWorkerAccess<'_>,
    task_id: Uuid,
    job_id: Uuid,
) -> ApplicationResult<Value> {
    execution::execute(
        access.workflow,
        access.jobs,
        access.artifacts,
        access.audit,
        access.pipeline,
        task_id,
        job_id,
    )
    .await
}

pub(crate) async fn finalize_successful_research(
    workflow: &dyn PredictionWorkflowPort,
    jobs: &dyn JobQueuePort,
    task: &P4FreezeTaskRecord,
) -> ApplicationResult<P4FreezeTaskRecord> {
    transitions::finalize_successful_research(workflow, jobs, task).await
}
