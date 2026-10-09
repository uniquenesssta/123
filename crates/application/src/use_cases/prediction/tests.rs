// Shared call recorder for the existing Prediction and P4 orchestration unit modules.
// Only test builds expose this probe; every unselected Port method fails immediately.
use crate::ports::lineup::{LineupPort, MatchCatalogPort};
use crate::ports::prediction::{
    ModelRunHistoryItem, ModelRunPort, PredictionInputPort, SerializedModelRun,
};
use crate::ports::rules::RuleRoutingPort;
use crate::ports::{PortError, PortErrorKind, PortResult};
use crate::{model_registry::ModelRegistry, ApplicationError, ApplicationService};
use async_trait::async_trait;
use chrono::DateTime;
use chrono::Utc;
use football_domain::{
    CompetitionBindingDraft, CompetitionBindingSummary, CompetitionKind, CompetitionProfile,
    EnqueueJobDraft, LineupDraft, LineupHistoryRemovalResult, LineupPairDraft, LineupPairRecord,
    LineupRecord, MatchContext, MatchDraft, MatchLineupChain, MatchRecord, P4FreezeReadiness,
    P4FreezeTaskRecord, P4FreezeTaskTransition, PredictionSummary, PreparedMatchPredictionInput,
    ResolvedCompetitionContext, RouteDecision, RouteRequest, RouteSource, RuleRouting,
    TeamMatchLineupHistoryItem,
};
use football_model_api::{
    ModelDescriptor, ModelError, ModelOutput, ModelRequest, ModelResult, PredictionModel,
};
use serde_json::json;
use serde_json::Value;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

#[derive(Default)]
pub(crate) struct ProbeState {
    pub(crate) planning_context: Option<football_domain::P4PlanningMatchContext>,
    pub(crate) planning_route: Option<RouteDecision>,
    pub(crate) planned_tasks: Vec<P4FreezeTaskRecord>,
    pub(crate) task_drafts: Vec<football_domain::P4FreezeTaskDraft>,
    pub(crate) schemas: Vec<football_domain::SchemaVersionRecord>,
    pub(crate) schema_reads: Vec<(String, String)>,
    pub(crate) planned_job_ids: std::collections::BTreeMap<String, Uuid>,
    pub(crate) job_results: Vec<(Uuid, String)>,
    pub(crate) job_failures: Vec<(Uuid, String)>,
    pub(crate) calls: Vec<&'static str>,
    pub(crate) failure: Option<&'static str>,
    pub(crate) failure_kind: Option<PortErrorKind>,
    pub(crate) failure_call_number: Option<usize>,
    pub(crate) scope_kind: Option<CompetitionKind>,
    pub(crate) match_record: Option<MatchRecord>,
    pub(crate) match_chain: Option<MatchLineupChain>,
    pub(crate) chain_requests: Vec<(Uuid, String, DateTime<Utc>)>,
    pub(crate) match_workspace: Option<football_domain::P4MatchWorkspace>,
    pub(crate) task_workspace: Option<football_domain::P4TaskWorkspace>,
    pub(crate) match_workspace_requests: Vec<Uuid>,
    pub(crate) task_workspace_requests: Vec<Uuid>,
    pub(crate) task: Option<P4FreezeTaskRecord>,
    pub(crate) readiness: Option<P4FreezeReadiness>,
    pub(crate) snapshot_id: Option<Uuid>,
    pub(crate) frozen_drafts: Vec<football_domain::PrematchSnapshotDraft>,
    pub(crate) freeze_routes: Option<Vec<football_domain::P4RoutedFact>>,
    pub(crate) provider_payload: Option<Value>,
    pub(crate) transitions: Vec<P4FreezeTaskTransition>,
    pub(crate) enqueues: Vec<EnqueueJobDraft>,
    pub(crate) queued_job_id: Option<Uuid>,
    pub(crate) prepared_input: Option<PreparedMatchPredictionInput>,
    pub(crate) input_requests: Vec<(Uuid, String, String, DateTime<Utc>)>,
    saved: Vec<(RouteDecision, ModelRequest, ModelOutput, i64)>,
    pub(crate) route_requests: Vec<RouteRequest>,
}

pub(crate) struct Probe {
    pub(crate) state: Mutex<ProbeState>,
    route: RouteDecision,
}

impl Probe {
    pub(crate) fn new() -> Self {
        Self {
            state: Mutex::new(ProbeState::default()),
            route: route_fixture(),
        }
    }
    pub(crate) fn call(&self, name: &'static str) -> PortResult<()> {
        let mut state = self.state.lock().unwrap();
        state.calls.push(name);
        if state.failure == Some(name)
            && state.failure_call_number.is_none_or(|number| {
                state.calls.iter().filter(|call| **call == name).count() == number
            })
        {
            return Err(PortError::new(
                state.failure_kind.unwrap_or(PortErrorKind::Unavailable),
                format!("injected {name}"),
            ));
        }
        Ok(())
    }
    pub(crate) fn calls(&self) -> Vec<&'static str> {
        self.state.lock().unwrap().calls.clone()
    }
    pub(crate) fn fail_at(&self, name: Option<&'static str>) {
        self.state.lock().unwrap().failure = name;
    }
}

pub(crate) fn route_fixture() -> RouteDecision {
    RouteDecision {
        source: RouteSource::CompetitionKindDefault,
        binding_id: None,
        rule_package_id: Uuid::from_u128(10),
        package_key: "test-package".into(),
        package_version: "test-rule-v1".into(),
        package_display_name: "test only".into(),
        model_id: "p4_league".into(),
        model_version_id: Uuid::from_u128(11),
        model_version: "test-model-v1".into(),
        parameter_set_id: Uuid::from_u128(12),
        parameter_version: "test-parameter-v1".into(),
        competition_profile_id: Uuid::from_u128(13),
        parameters: json!({"test_parameter": 7}),
        routing: RuleRouting {
            model_id: "p4_league".into(),
            model_version: "test-model-v1".into(),
            parameter_version: "test-parameter-v1".into(),
            priority: 0,
            activate_as_type_default: false,
            supported_snapshot_types: vec!["T-1h".into()],
        },
        competition_profile: CompetitionProfile {
            profile_id: "test-profile".into(),
            name: "test only".into(),
            competition_kind: CompetitionKind::League,
            normal_time_minutes: 90,
            extra_time_possible: false,
            penalties_possible: false,
            two_legged: false,
            neutral_venue: false,
            metadata: json!({}),
        },
        feature_requirements: json!({}),
        output_contract: json!({}),
        priority: 0,
        reason: json!({}),
    }
}

pub(crate) fn command_fixture() -> crate::PredictionCommand {
    crate::PredictionCommand {
        match_input: json!({"kickoff_time": "2026-09-30T12:00:00Z", "team_a": {"name": "Home"}, "team_b": {"name": "Away"}}),
        snapshot_type: "T-1h".into(),
        competition_id: Some(Uuid::from_u128(21)),
        season_id: Some(Uuid::from_u128(22)),
        stage_id: Some(Uuid::from_u128(23)),
        competition_kind: CompetitionKind::League,
        model_family: "P4_LEAGUE".into(),
        explicit_rule_package_id: None,
    }
}

// This model is an orchestration-only fake. It proves neither model accuracy nor private provider availability.
struct ProbeModel(Arc<Probe>);
impl PredictionModel for ProbeModel {
    fn descriptor(&self) -> ModelDescriptor {
        ModelDescriptor {
            model_id: "p4_league".into(),
            display_name: "orchestration test fake".into(),
            engine_version: "test-fake".into(),
            supported_competitions: vec![CompetitionKind::League],
            input_schema_version: "test".into(),
            output_schema_version: "test".into(),
        }
    }
    fn supports(&self, context: &MatchContext) -> bool {
        assert_eq!(context.competition_kind, CompetitionKind::League);
        self.0.call("model_supports").is_ok()
    }
    fn validate_input(&self, _input: &Value) -> ModelResult<()> {
        Ok(())
    }
    fn validate_parameters(&self, _parameters: &Value) -> ModelResult<()> {
        Ok(())
    }
    fn predict(&self, request: &ModelRequest) -> ModelResult<ModelOutput> {
        self.0
            .call("predict")
            .map_err(|e| ModelError::Calculation(e.to_string()))?;
        assert_eq!(request.identity.model_version, "test-model-v1");
        assert_eq!(request.identity.parameter_version, "test-parameter-v1");
        assert_eq!(request.parameters, json!({"test_parameter": 7}));
        assert_eq!(request.input["match_id"], request.context.match_key);
        Ok(ModelOutput {
            identity: request.identity.clone(),
            summary: PredictionSummary {
                home_win: 0.4,
                draw: 0.3,
                away_win: 0.3,
                btts: None,
                over_2_5: None,
            },
            payload: self
                .0
                .state
                .lock()
                .unwrap()
                .provider_payload
                .clone()
                .unwrap_or_else(|| json!({"test_only": true})),
            explanation: json!({}),
        })
    }
}
pub(crate) fn probe_registry(port: &Arc<Probe>) -> ModelRegistry {
    let mut registry = ModelRegistry::new();
    registry.register(Arc::new(ProbeModel(Arc::clone(port))));
    registry
}

#[tokio::test]
async fn formal_prediction_saves_normalized_input_and_routed_identity_once() {
    let port = Arc::new(Probe::new());
    let command = command_fixture();
    let original = command.match_input.clone();
    let result = super::execute_prediction::execute(port.as_ref(), &probe_registry(&port), command)
        .await
        .unwrap();
    assert_eq!(result.run_id, Uuid::from_u128(99));
    assert_eq!(
        port.calls(),
        ["scope", "route", "model_supports", "predict", "save_run"]
    );
    let state = port.state.lock().unwrap();
    let (route, request, output, duration) = &state.saved[0];
    assert_eq!(state.saved.len(), 1);
    assert_eq!(route.rule_package_id, Uuid::from_u128(10));
    assert_eq!(
        serde_json::to_value(&request.identity).unwrap(),
        serde_json::to_value(&output.identity).unwrap()
    );
    assert_eq!(
        request.identity.rule_package_version.as_deref(),
        Some("test-rule-v1")
    );
    assert_eq!(request.snapshot_type, "T-1h");
    assert_eq!(request.context.competition_id, Some(Uuid::from_u128(21)));
    assert_eq!(request.input["team_a"], original["team_a"]);
    assert!(request.input["match_id"]
        .as_str()
        .unwrap()
        .starts_with("SIM-"));
    assert_eq!(*duration, result.duration_ms);
    assert!(*duration >= 0);
    let routed = &state.route_requests[0];
    assert_eq!(routed.preferred_model_family.as_deref(), Some("p4"));
    assert_eq!(routed.preferred_model_id.as_deref(), Some("p4_league"));
    assert_eq!(routed.season_id, Some(Uuid::from_u128(22)));
    assert_eq!(routed.stage_id, Some(Uuid::from_u128(23)));
}

#[tokio::test]
async fn repeated_shadow_prediction_never_writes_model_history() {
    let port = Arc::new(Probe::new());
    let registry = probe_registry(&port);
    for _ in 0..2 {
        let result = super::execute_prediction::execute_internal(
            port.as_ref(),
            &registry,
            command_fixture(),
            false,
        )
        .await
        .unwrap();
        assert!(result.run_id.is_nil());
    }
    assert_eq!(
        port.calls(),
        [
            "scope",
            "route",
            "model_supports",
            "predict",
            "scope",
            "route",
            "model_supports",
            "predict"
        ]
    );
    assert!(port.state.lock().unwrap().saved.is_empty());
}

#[tokio::test]
async fn prediction_errors_stop_every_later_side_effect() {
    for (failure, expected) in [
        ("scope", vec!["scope"]),
        ("route", vec!["scope", "route"]),
        ("model_supports", vec!["scope", "route", "model_supports"]),
        (
            "predict",
            vec!["scope", "route", "model_supports", "predict"],
        ),
    ] {
        let port = Arc::new(Probe::new());
        port.fail_at(Some(failure));
        let error = super::execute_prediction::execute(
            port.as_ref(),
            &probe_registry(&port),
            command_fixture(),
        )
        .await
        .unwrap_err();
        if matches!(failure, "scope" | "route") {
            assert!(matches!(error, ApplicationError::Port(_)));
        } else {
            assert!(matches!(error, ApplicationError::Model(_)));
        }
        assert_eq!(port.calls(), expected, "failure at {failure}");
        assert!(port.state.lock().unwrap().saved.is_empty());
    }
}

#[tokio::test]
async fn failed_run_save_is_propagated_without_success_or_automatic_retry() {
    for kind in [
        PortErrorKind::Unavailable,
        PortErrorKind::NotFound,
        PortErrorKind::Conflict,
        PortErrorKind::InvalidState,
        PortErrorKind::Serialization,
        PortErrorKind::Infrastructure,
    ] {
        let port = Arc::new(Probe::new());
        port.fail_at(Some("save_run"));
        port.state.lock().unwrap().failure_kind = Some(kind);
        let error = super::execute_prediction::execute(
            port.as_ref(),
            &probe_registry(&port),
            command_fixture(),
        )
        .await
        .unwrap_err();
        assert!(
            matches!(error, ApplicationError::Port(PortError { kind: actual, message })
            if actual == kind && message == "injected save_run")
        );
        assert_eq!(
            port.calls(),
            ["scope", "route", "model_supports", "predict", "save_run"]
        );
        assert!(port.state.lock().unwrap().saved.is_empty());
    }
}

#[tokio::test]
async fn stale_input_route_and_unsupported_snapshot_never_call_model_or_save() {
    for stale_route in [true, false] {
        let port = Arc::new(Probe::new());
        let mut command = command_fixture();
        if stale_route {
            command.match_input["input_audit"] =
                json!({"manifest": {"route_identity": {"stale": true}}});
        } else {
            command.snapshot_type = "actual".into();
        }
        let error =
            super::execute_prediction::execute(port.as_ref(), &probe_registry(&port), command)
                .await
                .unwrap_err();
        assert!(matches!(error, ApplicationError::Validation(_)));
        assert_eq!(port.calls(), ["scope", "route"]);
        assert!(port.state.lock().unwrap().saved.is_empty());
    }
}

#[tokio::test]
async fn public_external_provider_unavailable_error_cannot_be_saved_as_success() {
    let port = Probe::new();
    let registry = ApplicationService::new().registry;
    let error = super::execute_prediction::execute(&port, &registry, command_fixture())
        .await
        .unwrap_err();
    assert!(matches!(error, ApplicationError::Model(message) if message.contains("未随仓库分发")));
    assert_eq!(port.calls(), ["scope", "route"]);
    assert!(port.state.lock().unwrap().saved.is_empty());
}

#[async_trait]
impl RuleRoutingPort for Probe {
    async fn create_competition_binding(
        &self,
        _draft: &CompetitionBindingDraft,
    ) -> PortResult<CompetitionBindingSummary> {
        panic!("forbidden Port call: create_competition_binding")
    }
    async fn list_competition_bindings(&self) -> PortResult<Vec<CompetitionBindingSummary>> {
        panic!("forbidden Port call: list_competition_bindings")
    }
    async fn ensure_type_default_binding(
        &self,
        _rule_package_id: Uuid,
        _competition_kind: CompetitionKind,
        _priority: i32,
        _label: &str,
    ) -> PortResult<()> {
        panic!("forbidden Port call: ensure_type_default_binding")
    }
    async fn resolve_competition_context(
        &self,
        competition_id: Option<Uuid>,
        season_id: Option<Uuid>,
        stage_id: Option<Uuid>,
        competition_kind: CompetitionKind,
    ) -> PortResult<ResolvedCompetitionContext> {
        self.call("scope")?;
        Ok(ResolvedCompetitionContext {
            competition_id,
            season_id,
            stage_id,
            competition_kind: self
                .state
                .lock()
                .unwrap()
                .scope_kind
                .unwrap_or(competition_kind),
        })
    }
    async fn resolve_route(&self, request: &RouteRequest) -> PortResult<RouteDecision> {
        self.call("route")?;
        self.state
            .lock()
            .unwrap()
            .route_requests
            .push(request.clone());
        Ok(self
            .state
            .lock()
            .unwrap()
            .planning_route
            .clone()
            .unwrap_or_else(|| self.route.clone()))
    }
}

#[async_trait]
impl ModelRunPort for Probe {
    async fn save_successful_run(
        &self,
        decision: &RouteDecision,
        request: &ModelRequest,
        output: &ModelOutput,
        duration_ms: i64,
    ) -> PortResult<Uuid> {
        self.call("save_run")?;
        self.state.lock().unwrap().saved.push((
            decision.clone(),
            request.clone(),
            output.clone(),
            duration_ms,
        ));
        Ok(Uuid::from_u128(99))
    }
    async fn hide_run_from_history(&self, _run_id: Uuid, _reason: Option<&str>) -> PortResult<()> {
        panic!("forbidden Port call: hide_run_from_history")
    }
    async fn list_recent_runs(&self, _limit: i64) -> PortResult<Vec<ModelRunHistoryItem>> {
        panic!("forbidden Port call: list_recent_runs")
    }
    async fn read_run_document(&self, _run_id: Uuid) -> PortResult<SerializedModelRun> {
        panic!("forbidden Port call: read_run_document")
    }
}

#[async_trait]
impl PredictionInputPort for Probe {
    async fn prepare_match_input(
        &self,
        match_id: Uuid,
        snapshot_type: &str,
        model_family: &str,
    ) -> PortResult<PreparedMatchPredictionInput> {
        self.call("prepare_input")?;
        let state = self.state.lock().unwrap();
        let prepared = state
            .prepared_input
            .clone()
            .expect("unexpected prediction input preparation");
        assert_eq!(prepared.match_record.id, match_id);
        assert_eq!(prepared.snapshot_type, snapshot_type);
        assert_eq!(model_family, "p4");
        Ok(prepared)
    }
    async fn prepare_match_input_at(
        &self,
        match_id: Uuid,
        snapshot_type: &str,
        model_family: &str,
        reference_time: DateTime<Utc>,
    ) -> PortResult<PreparedMatchPredictionInput> {
        self.call("prepare_input_at")?;
        let mut state = self.state.lock().unwrap();
        state.input_requests.push((
            match_id,
            snapshot_type.to_string(),
            model_family.to_string(),
            reference_time,
        ));
        Ok(state
            .prepared_input
            .clone()
            .expect("unexpected prediction input preparation at assessed time"))
    }
}

#[async_trait]
impl MatchCatalogPort for Probe {
    async fn create_match(&self, _draft: &MatchDraft) -> PortResult<MatchRecord> {
        panic!("forbidden Port call: create_match")
    }
    async fn delete_match(&self, _match_id: Uuid) -> PortResult<()> {
        panic!("forbidden Port call: delete_match")
    }
    async fn read_match(&self, _match_id: Uuid) -> PortResult<MatchRecord> {
        self.call("read_match")?;
        Ok(self
            .state
            .lock()
            .unwrap()
            .match_record
            .clone()
            .expect("unselected Port call: read_match"))
    }
}

#[async_trait]
impl LineupPort for Probe {
    async fn create_lineup(&self, _draft: &LineupDraft) -> PortResult<LineupRecord> {
        panic!("forbidden Port call: create_lineup")
    }
    async fn create_lineup_pair(&self, _draft: &LineupPairDraft) -> PortResult<LineupPairRecord> {
        panic!("forbidden Port call: create_lineup_pair")
    }
    async fn list_lineups(
        &self,
        _match_id: Option<Uuid>,
        _limit: u32,
    ) -> PortResult<Vec<LineupRecord>> {
        panic!("forbidden Port call: list_lineups")
    }
    async fn read_lineup(&self, _lineup_id: Uuid) -> PortResult<LineupRecord> {
        panic!("forbidden Port call: read_lineup")
    }
    async fn remove_history(
        &self,
        _lineup_id: Uuid,
        _reason: Option<&str>,
    ) -> PortResult<LineupHistoryRemovalResult> {
        panic!("forbidden Port call: remove_history")
    }
    async fn read_match_chain(
        &self,
        _match_id: Uuid,
        _snapshot_type: &str,
    ) -> PortResult<MatchLineupChain> {
        panic!("forbidden Port call: read_match_chain")
    }
    async fn read_match_chain_at(
        &self,
        match_id: Uuid,
        snapshot_type: &str,
        reference_time: DateTime<Utc>,
    ) -> PortResult<MatchLineupChain> {
        self.call("read_chain_at")?;
        let mut state = self.state.lock().unwrap();
        state
            .chain_requests
            .push((match_id, snapshot_type.to_string(), reference_time));
        Ok(state
            .match_chain
            .clone()
            .expect("unselected Port call: read_match_chain_at"))
    }
    async fn list_team_match_lineups(
        &self,
        _team_id: Uuid,
        _limit: u32,
    ) -> PortResult<Vec<TeamMatchLineupHistoryItem>> {
        panic!("forbidden Port call: list_team_match_lineups")
    }
}
