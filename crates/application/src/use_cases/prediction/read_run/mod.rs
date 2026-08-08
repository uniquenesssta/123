use crate::ports::prediction::ModelRunPort;
use crate::ApplicationResult;
use serde_json::Value;
use uuid::Uuid;

pub(crate) async fn execute<P: ModelRunPort + ?Sized>(
    port: &P,
    run_id: Uuid,
) -> ApplicationResult<Value> {
    let document = port.read_run_document(run_id).await?;
    Ok(serde_json::from_str(&document.json)?)
}
