use super::route_model_request::default_fixture_request;
use crate::model_registry::{prediction_model_adapter, ModelRegistry};
use crate::model_shell::P4_MODEL_ID;
use crate::ApplicationResult;
use football_model_api::ModelOutput;

pub(crate) fn execute(registry: &ModelRegistry) -> ApplicationResult<ModelOutput> {
    let model = prediction_model_adapter::registered_model(registry, P4_MODEL_ID)?;
    let request = default_fixture_request()?;
    prediction_model_adapter::predict(model.as_ref(), &request)
}
