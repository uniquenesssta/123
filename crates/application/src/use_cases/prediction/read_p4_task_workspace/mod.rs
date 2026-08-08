use crate::ports::prediction::PredictionWorkflowPort;
use crate::ApplicationResult;
use football_domain::P4TaskWorkspace;
use uuid::Uuid;

pub(crate) async fn execute<P: PredictionWorkflowPort + ?Sized>(
    port: &P,
    task_id: Uuid,
) -> ApplicationResult<P4TaskWorkspace> {
    Ok(port.read_task_workspace(task_id).await?)
}
