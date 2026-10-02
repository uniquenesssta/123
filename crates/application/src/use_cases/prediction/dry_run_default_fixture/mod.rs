use super::route_model_request::default_fixture_request;
use crate::model_registry::ModelRegistry;
use crate::model_shell::P4_MODEL_ID;
use crate::{ApplicationError, ApplicationResult};
use football_model_api::ModelOutput;

pub(crate) fn execute(registry: &ModelRegistry) -> ApplicationResult<ModelOutput> {
    let model = registry
        .get(P4_MODEL_ID)
        .ok_or_else(|| ApplicationError::ModelNotFound(P4_MODEL_ID.to_string()))?;
    let request = default_fixture_request()?;
    model
        .predict(&request)
        .map_err(|error| ApplicationError::Model(error.to_string()))
}
