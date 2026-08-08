use crate::ports::prediction::PredictionWorkflowPort;
use crate::ApplicationResult;
use football_domain::P4FreezeTaskRecord;
use uuid::Uuid;

pub(crate) async fn execute<P: PredictionWorkflowPort + ?Sized>(
    port: &P,
    match_id: Option<Uuid>,
    limit: u32,
) -> ApplicationResult<Vec<P4FreezeTaskRecord>> {
    Ok(port.list_freeze_tasks(match_id, limit).await?)
}
