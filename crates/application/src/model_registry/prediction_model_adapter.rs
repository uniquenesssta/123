use super::ModelRegistry;
use crate::{ApplicationError, ApplicationResult};
use football_domain::{CompetitionKind, MatchContext};
use football_model_api::{ModelOutput, ModelRequest, PredictionModel};
use std::sync::Arc;
use std::time::{Duration, Instant};

pub(crate) fn registered_model(
    registry: &ModelRegistry,
    model_id: &str,
) -> ApplicationResult<Arc<dyn PredictionModel>> {
    registry
        .get(model_id)
        .ok_or_else(|| ApplicationError::ModelNotFound(model_id.to_string()))
}

pub(crate) fn supported_model(
    registry: &ModelRegistry,
    model_id: &str,
    context: &MatchContext,
    scope_kind: CompetitionKind,
) -> ApplicationResult<Arc<dyn PredictionModel>> {
    let model = registered_model(registry, model_id)?;
    if !model.supports(context) {
        return Err(ApplicationError::Model(format!(
            "模型 {} 不支持赛事类型 {}",
            model.descriptor().display_name,
            scope_kind.as_str()
        )));
    }
    Ok(model)
}

pub(crate) fn predict(
    model: &dyn PredictionModel,
    request: &ModelRequest,
) -> ApplicationResult<ModelOutput> {
    model
        .predict(request)
        .map_err(|error| ApplicationError::Model(error.to_string()))
}

pub(crate) fn execute(
    model: &dyn PredictionModel,
    request: &ModelRequest,
) -> ApplicationResult<(ModelOutput, i64)> {
    let started = Instant::now();
    let output = predict(model, request)?;
    let duration_ms = elapsed_milliseconds(started.elapsed());
    Ok((output, duration_ms))
}

fn elapsed_milliseconds(elapsed: Duration) -> i64 {
    elapsed.as_millis().min(i64::MAX as u128) as i64
}

#[cfg(test)]
mod tests;
