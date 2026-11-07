use super::*;
use crate::default_rule_package_template;
use crate::{ApplicationError, ApplicationService};
use football_domain::{CompetitionKind, MatchContext};
use football_model_api::{
    ModelDescriptor, ModelError, ModelOutput, ModelRequest, ModelResult, PredictionModel,
};
use serde_json::{json, Value};
use std::sync::Arc;

struct InjectedProvider;

impl PredictionModel for InjectedProvider {
    fn descriptor(&self) -> ModelDescriptor {
        ModelDescriptor {
            model_id: "p4".into(),
            display_name: "Explicit host provider".into(),
            engine_version: "injected-test-provider".into(),
            supported_competitions: CompetitionKind::ALL.to_vec(),
            input_schema_version: "test.input.v1".into(),
            output_schema_version: "test.output.v1".into(),
        }
    }
    fn supports(&self, _: &MatchContext) -> bool {
        true
    }
    fn validate_input(&self, input: &Value) -> ModelResult<()> {
        if input["host_input"] == true {
            Ok(())
        } else {
            Err(ModelError::InvalidInput("host input required".into()))
        }
    }
    fn validate_parameters(&self, parameters: &Value) -> ModelResult<()> {
        if parameters["runtime_marker"] == true {
            Ok(())
        } else {
            Err(ModelError::InvalidParameters(
                "host parameters required".into(),
            ))
        }
    }
    fn predict(&self, _: &ModelRequest) -> ModelResult<ModelOutput> {
        Err(ModelError::Calculation("host provider reached".into()))
    }
}

fn runtime() -> ApplicationModelRuntime {
    let mut registry = ModelRegistry::new();
    registry.register(Arc::new(InjectedProvider));
    let mut template = default_rule_package_template();
    template.parameters["runtime_marker"] = json!(true);
    let mut request =
        crate::use_cases::prediction::route_model_request::default_fixture_request().unwrap();
    request.parameters = template.parameters.clone();
    request.input = json!({"host_input":true});
    ApplicationModelRuntime {
        registry,
        default_prediction_request: request,
        startup_rule_packages: vec![template.clone()],
        default_rule_package_template: template,
    }
}

#[tokio::test]
async fn supplied_provider_and_template_reach_application_facades() {
    let application = ApplicationService::with_model_runtime(runtime()).unwrap();
    let bootstrap = application.bootstrap().await.unwrap();
    assert!(!bootstrap.database_configured);
    assert_eq!(bootstrap.default_match["host_input"], true);
    assert_eq!(bootstrap.models.len(), 1);
    assert_eq!(bootstrap.models[0].engine_version, "injected-test-provider");
    assert_eq!(
        bootstrap.default_rule_package.parameters["runtime_marker"],
        true
    );
    assert!(
        matches!(application.dry_run_default_fixture(), Err(ApplicationError::Model(message)) if message.contains("host provider reached"))
    );
}

#[test]
fn mismatched_provider_parameters_fail_before_database_connection() {
    let mut runtime = runtime();
    runtime.startup_rule_packages[0].parameters["runtime_marker"] = json!(false);
    assert!(
        matches!(ApplicationService::with_model_runtime(runtime), Err(ApplicationError::Model(message)) if message.contains("host parameters required"))
    );
}

#[test]
fn missing_provider_does_not_fall_back_to_public_stub() {
    let mut runtime = runtime();
    runtime.registry = ModelRegistry::new();
    assert!(
        matches!(ApplicationService::with_model_runtime(runtime), Err(ApplicationError::ModelNotFound(model)) if model == "p4")
    );
}

#[test]
fn invalid_default_request_fails_before_database_connection() {
    let mut runtime = runtime();
    runtime.default_prediction_request.input = json!({});
    assert!(
        matches!(ApplicationService::with_model_runtime(runtime), Err(ApplicationError::Model(message)) if message.contains("host input required"))
    );
}
