use super::PredictionAccess;
use crate::model_registry::ModelRegistry;
use crate::{ApplicationResult, PredictionExecution, StoredMatchPredictionCommand};

pub(crate) async fn execute_formal<P: PredictionAccess + ?Sized>(
    port: &P,
    registry: &ModelRegistry,
    command: StoredMatchPredictionCommand,
) -> ApplicationResult<PredictionExecution> {
    execute_with_mode(port, registry, command, true).await
}

pub(crate) async fn execute_shadow<P: PredictionAccess + ?Sized>(
    port: &P,
    registry: &ModelRegistry,
    command: StoredMatchPredictionCommand,
) -> ApplicationResult<PredictionExecution> {
    execute_with_mode(port, registry, command, false).await
}

async fn execute_with_mode<P: PredictionAccess + ?Sized>(
    port: &P,
    registry: &ModelRegistry,
    command: StoredMatchPredictionCommand,
    persist_run: bool,
) -> ApplicationResult<PredictionExecution> {
    let input = super::build_input::execute(port, registry, command, persist_run).await?;
    super::execute_prediction::execute_internal(port, registry, input, persist_run).await
}
