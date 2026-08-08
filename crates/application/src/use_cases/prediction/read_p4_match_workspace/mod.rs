use crate::ports::prediction::PredictionWorkflowPort;
use crate::ApplicationResult;
use football_domain::P4MatchWorkspace;
use uuid::Uuid;

pub(crate) async fn execute<P: PredictionWorkflowPort + ?Sized>(
    port: &P,
    match_id: Uuid,
) -> ApplicationResult<P4MatchWorkspace> {
    Ok(port.read_match_workspace(match_id).await?)
}
