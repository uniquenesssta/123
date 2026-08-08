use crate::ports::prediction::PredictionWorkflowPort;
use crate::ApplicationResult;
use football_domain::P4FreezeReadiness;
use uuid::Uuid;

pub(crate) async fn execute<P: PredictionWorkflowPort + ?Sized>(
    port: &P,
    task_id: Uuid,
) -> ApplicationResult<P4FreezeReadiness> {
    Ok(port.freeze_readiness(task_id).await?)
}
