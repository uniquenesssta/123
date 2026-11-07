use super::route_model_request::default_fixture_request;
use crate::model_registry::{prediction_model_adapter, ModelRegistry};
use crate::model_shell::P4_MODEL_ID;
use crate::ApplicationResult;
use football_model_api::{ModelOutput, ModelRequest};

pub(crate) fn execute(
    registry: &ModelRegistry,
    supplied: Option<&ModelRequest>,
) -> ApplicationResult<ModelOutput> {
    let model_id = supplied
        .map(|request| request.identity.model_id.as_str())
        .unwrap_or(P4_MODEL_ID);
    let model = prediction_model_adapter::registered_model(registry, model_id)?;
    let fallback;
    let request = if let Some(request) = supplied {
        request
    } else {
        fallback = default_fixture_request()?;
        &fallback
    };
    prediction_model_adapter::predict(model.as_ref(), request)
}
