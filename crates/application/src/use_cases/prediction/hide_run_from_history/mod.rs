use crate::ports::prediction::ModelRunPort;
use crate::ApplicationResult;
use uuid::Uuid;

pub(crate) async fn execute<P: ModelRunPort + ?Sized>(
    port: &P,
    run_id: Uuid,
    reason: Option<String>,
) -> ApplicationResult<()> {
    Ok(port
        .hide_run_from_history(run_id, reason.as_deref())
        .await?)
}
