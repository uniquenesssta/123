use crate::ports::prediction::{ModelRunHistoryItem, ModelRunPort};
use crate::ApplicationResult;

pub(crate) async fn execute<P: ModelRunPort + ?Sized>(
    port: &P,
    limit: i64,
) -> ApplicationResult<Vec<ModelRunHistoryItem>> {
    Ok(port.list_recent_runs(limit).await?)
}
