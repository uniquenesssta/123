use crate::ports::prediction::PredictionWorkflowPort;
use crate::ApplicationResult;
use football_domain::P4FreezeTaskEventRecord;
use uuid::Uuid;

pub(crate) async fn execute<P: PredictionWorkflowPort + ?Sized>(
    port: &P,
    task_id: Uuid,
) -> ApplicationResult<Vec<P4FreezeTaskEventRecord>> {
    Ok(port.list_freeze_task_events(task_id).await?)
}
