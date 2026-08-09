use crate::ports::{
    analytics::JobQueuePort,
    prediction::PredictionWorkflowPort,
    research::{ResearchArtifactPort, ResearchManualConflictPort},
};
use crate::ApplicationResult;
use chrono::Utc;
use football_domain::{P4TaskWorkspace, ResolveP4ConflictCommand};

mod decision;
mod reconciliation;

pub(crate) struct P4ManualConflictAccess<'a> {
    pub workflow: &'a dyn PredictionWorkflowPort,
    pub jobs: &'a dyn JobQueuePort,
    pub artifacts: &'a dyn ResearchArtifactPort,
    pub manual: &'a dyn ResearchManualConflictPort,
}

pub(crate) async fn resolve(
    access: P4ManualConflictAccess<'_>,
    command: ResolveP4ConflictCommand,
) -> ApplicationResult<P4TaskWorkspace> {
    let workspace = access.workflow.read_task_workspace(command.task_id).await?;
    let prepared = decision::prepare(&workspace, command, Utc::now())?;
    let (task_id, research_run_id) = match prepared {
        decision::PreparedDecision::Existing {
            task_id,
            research_run_id,
        } => (task_id, research_run_id),
        decision::PreparedDecision::Append {
            task_id,
            research_run_id,
            draft,
        } => {
            access
                .manual
                .append_manual_route_override(draft.as_ref())
                .await?;
            (task_id, research_run_id)
        }
    };

    reconciliation::reconcile(&access, task_id, research_run_id).await?;
    Ok(access.workflow.read_task_workspace(task_id).await?)
}
