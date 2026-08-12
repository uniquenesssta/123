use super::PredictionService;
use crate::composition::{model_run_list_item_from_port, ModelRunListItem};
use crate::use_cases::prediction::PredictionAccess;
use crate::ApplicationResult;

pub(crate) async fn list_recent_runs<P: PredictionAccess + ?Sized>(
    service: &PredictionService,
    port: &P,
    limit: i64,
) -> ApplicationResult<Vec<ModelRunListItem>> {
    service
        .list_recent_runs(port, limit)
        .await?
        .into_iter()
        .map(model_run_list_item_from_port)
        .collect::<Result<Vec<_>, _>>()
        .map_err(Into::into)
}
