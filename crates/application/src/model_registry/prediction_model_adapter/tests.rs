use super::*;
use crate::use_cases::prediction::route_model_request::default_fixture_request;
use crate::use_cases::prediction::tests::{command_fixture, Probe};
use football_domain::PredictionSummary;
use football_model_api::{ModelDescriptor, ModelError, ModelResult};
use serde_json::{json, Value};
use std::sync::Mutex;

#[derive(Default)]
struct Calls {
    contexts: Vec<Value>,
    requests: Vec<Value>,
    descriptors: usize,
    input_validations: usize,
    parameter_validations: usize,
}

struct RecordingModel {
    model_id: &'static str,
    calls: Arc<Mutex<Calls>>,
    supported: bool,
    failure: Option<fn() -> ModelError>,
}

impl PredictionModel for RecordingModel {
    fn descriptor(&self) -> ModelDescriptor {
        self.calls.lock().unwrap().descriptors += 1;
        ModelDescriptor {
            model_id: self.model_id.into(),
            display_name: "adapter test provider".into(),
            engine_version: "test".into(),
            supported_competitions: vec![CompetitionKind::League],
            input_schema_version: "test".into(),
            output_schema_version: "test".into(),
        }
    }

    fn supports(&self, context: &MatchContext) -> bool {
        self.calls
            .lock()
            .unwrap()
            .contexts
            .push(serde_json::to_value(context).unwrap());
        self.supported
    }

    fn validate_input(&self, _input: &Value) -> ModelResult<()> {
        self.calls.lock().unwrap().input_validations += 1;
        Err(ModelError::InvalidInput(
            "unexpected adapter validation".into(),
        ))
    }

    fn validate_parameters(&self, _parameters: &Value) -> ModelResult<()> {
        self.calls.lock().unwrap().parameter_validations += 1;
        Err(ModelError::InvalidParameters(
            "unexpected adapter validation".into(),
        ))
    }

    fn predict(&self, request: &ModelRequest) -> ModelResult<ModelOutput> {
        self.calls
            .lock()
            .unwrap()
            .requests
            .push(serde_json::to_value(request).unwrap());
        if let Some(failure) = self.failure {
            return Err(failure());
        }
        Ok(provider_output())
    }
}

// An orchestration-only provider fake, not private engine or probability regression evidence.
fn provider_output() -> ModelOutput {
    let mut identity = default_fixture_request().unwrap().identity;
    identity.model_version = "provider-returned-version".into();
    ModelOutput {
        identity,
        summary: PredictionSummary {
            home_win: 0.37,
            draw: 0.31,
            away_win: 0.32,
            btts: Some(0.53),
            over_2_5: Some(0.49),
        },
        payload: json!({"provider_only": {"matrix": [[0.12, 0.07], [0.21, 0.16]]}}),
        explanation: json!({"provider_only": ["unchanged", {"evidence": null}]}),
    }
}

fn registry_with(
    model_id: &'static str,
    supported: bool,
    failure: Option<fn() -> ModelError>,
) -> (ModelRegistry, Arc<Mutex<Calls>>, Arc<dyn PredictionModel>) {
    let calls = Arc::new(Mutex::new(Calls::default()));
    let model: Arc<dyn PredictionModel> = Arc::new(RecordingModel {
        model_id,
        calls: Arc::clone(&calls),
        supported,
        failure,
    });
    let mut registry = ModelRegistry::new();
    registry.register(Arc::clone(&model));
    *calls.lock().unwrap() = Calls::default();
    (registry, calls, model)
}

#[test]
fn registered_lookup_keeps_exact_provider_identity_and_missing_error() {
    let (registry, calls, original) = registry_with("p4_league", true, None);
    let model = registered_model(&registry, "p4_league").unwrap();
    assert!(Arc::ptr_eq(&model, &original));
    for id in ["p4_missing", " P4_LEAGUE ", "p7_league"] {
        let error = registered_model(&registry, id).err().unwrap();
        assert!(matches!(error, ApplicationError::ModelNotFound(value) if value == id));
    }
    let calls = calls.lock().unwrap();
    assert!(calls.contexts.is_empty() && calls.requests.is_empty());
    assert_eq!(calls.descriptors, 0);
}

#[test]
fn supported_lookup_uses_actual_context_and_original_scope_error() {
    let mut context = default_fixture_request().unwrap().context;
    context.competition_kind = CompetitionKind::KnockoutSingleLeg;
    let expected = serde_json::to_value(&context).unwrap();
    for supported in [true, false] {
        let (registry, calls, original) = registry_with("p4_league", supported, None);
        let result = supported_model(&registry, "p4_league", &context, CompetitionKind::League);
        if supported {
            assert!(Arc::ptr_eq(&result.unwrap(), &original));
        } else {
            assert!(
                matches!(result.err().unwrap(), ApplicationError::Model(message)
                if message == format!("模型 adapter test provider 不支持赛事类型 {}", CompetitionKind::League.as_str()))
            );
        }
        let calls = calls.lock().unwrap();
        assert_eq!(calls.contexts, [expected.clone()]);
        assert!(calls.requests.is_empty());
        assert_eq!(calls.descriptors, usize::from(!supported));
    }
}

#[test]
fn timed_execution_preserves_complete_request_and_provider_output() {
    let (_, calls, model) = registry_with("p4_league", true, None);
    let mut request = default_fixture_request().unwrap();
    request.input["adapter_identity"] = json!({"source": "unchanged", "cutoff": null});
    request.parameters = json!({"nested": [7, null, {"exact": "  value  "}]});
    let original = serde_json::to_value(&request).unwrap();
    let (output, duration_ms) = execute(model.as_ref(), &request).unwrap();
    assert_eq!(serde_json::to_value(&request).unwrap(), original);
    assert_eq!(
        serde_json::to_value(output).unwrap(),
        serde_json::to_value(provider_output()).unwrap()
    );
    assert!(duration_ms >= 0);
    let calls = calls.lock().unwrap();
    assert_eq!(calls.requests, [original]);
    assert!(calls.contexts.is_empty());
    assert_eq!(
        (
            calls.descriptors,
            calls.input_validations,
            calls.parameter_validations
        ),
        (0, 0, 0)
    );
}

#[test]
fn model_errors_keep_all_messages_without_retry_or_fallback() {
    let failures: [fn() -> ModelError; 5] = [
        || ModelError::InvalidInput("input detail".into()),
        || ModelError::InvalidParameters("parameters detail".into()),
        || ModelError::Calculation("calculation detail".into()),
        || ModelError::Serialization("serialization detail".into()),
        || ModelError::Unavailable("public provider unavailable".into()),
    ];
    for failure in failures {
        for timed in [false, true] {
            let (_, calls, model) = registry_with("p4_league", true, Some(failure));
            let request = default_fixture_request().unwrap();
            let error = if timed {
                execute(model.as_ref(), &request).unwrap_err()
            } else {
                predict(model.as_ref(), &request).unwrap_err()
            };
            assert!(
                matches!(error, ApplicationError::Model(message) if message == failure().to_string())
            );
            let calls = calls.lock().unwrap();
            assert_eq!(calls.requests.len(), 1);
            assert!(calls.contexts.is_empty());
            assert_eq!(
                (calls.input_validations, calls.parameter_validations),
                (0, 0)
            );
        }
    }
}

#[test]
fn elapsed_milliseconds_truncate_and_saturate_without_overflow() {
    assert_eq!(elapsed_milliseconds(Duration::ZERO), 0);
    assert_eq!(elapsed_milliseconds(Duration::from_nanos(999_999)), 0);
    assert_eq!(elapsed_milliseconds(Duration::from_nanos(1_999_999)), 1);
    assert_eq!(
        elapsed_milliseconds(Duration::from_millis(i64::MAX as u64)),
        i64::MAX
    );
    assert_eq!(
        elapsed_milliseconds(Duration::from_millis(i64::MAX as u64 + 1)),
        i64::MAX
    );
    assert_eq!(elapsed_milliseconds(Duration::MAX), i64::MAX);
}

#[test]
fn default_dry_run_keeps_request_and_omits_supports_and_validation() {
    let (registry, calls, _) = registry_with(crate::model_shell::P4_MODEL_ID, false, None);
    let output = crate::use_cases::prediction::dry_run_default_fixture::execute(&registry).unwrap();
    assert_eq!(
        serde_json::to_value(output).unwrap(),
        serde_json::to_value(provider_output()).unwrap()
    );
    let calls = calls.lock().unwrap();
    assert_eq!(
        calls.requests,
        [serde_json::to_value(default_fixture_request().unwrap()).unwrap()]
    );
    assert!(calls.contexts.is_empty());
    assert_eq!(
        (
            calls.descriptors,
            calls.input_validations,
            calls.parameter_validations
        ),
        (0, 0, 0)
    );
    assert!(
        matches!(crate::use_cases::prediction::dry_run_default_fixture::execute(&ModelRegistry::new()),
        Err(ApplicationError::ModelNotFound(id)) if id == crate::model_shell::P4_MODEL_ID)
    );
}

#[tokio::test]
async fn routed_failures_preserve_precedence_and_stop_history_writes() {
    // The original support check precedes request assembly and input audit validation.
    let port = Probe::new();
    let (registry, calls, _) = registry_with("p4_league", false, None);
    let mut command = command_fixture();
    command.match_input["input_audit"] = json!({"manifest": "invalid"});
    let error =
        crate::use_cases::prediction::execute_prediction::execute(&port, &registry, command)
            .await
            .unwrap_err();
    assert!(
        matches!(error, ApplicationError::Model(message) if message.contains("不支持赛事类型"))
    );
    assert_eq!(port.calls(), ["scope", "route"]);
    assert_eq!(calls.lock().unwrap().contexts.len(), 1);
    assert!(calls.lock().unwrap().requests.is_empty());

    // A routed but unregistered model must fail before the malformed audit, without fallback.
    let port = Probe::new();
    let mut command = command_fixture();
    command.model_family = "p4".into();
    command.match_input["input_audit"] = json!({"manifest": "invalid"});
    let error = crate::use_cases::prediction::execute_prediction::execute(
        &port,
        &ModelRegistry::new(),
        command,
    )
    .await
    .unwrap_err();
    assert!(matches!(error, ApplicationError::ModelNotFound(id) if id == "p4_league"));
    assert_eq!(port.calls(), ["scope", "route"]);

    // Provider failure is propagated after successful routing; no save or automatic retry follows.
    let port = Probe::new();
    let (registry, calls, _) = registry_with(
        "p4_league",
        true,
        Some(|| ModelError::Unavailable("offline".into())),
    );
    let error = crate::use_cases::prediction::execute_prediction::execute(
        &port,
        &registry,
        command_fixture(),
    )
    .await
    .unwrap_err();
    assert!(
        matches!(error, ApplicationError::Model(message) if message == ModelError::Unavailable("offline".into()).to_string())
    );
    assert_eq!(port.calls(), ["scope", "route"]);
    assert_eq!(calls.lock().unwrap().requests.len(), 1);
}
