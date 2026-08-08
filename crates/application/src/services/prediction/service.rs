use crate::model_registry::ModelRegistry;
use crate::ports::prediction::ModelRunHistoryItem;
use crate::use_cases::prediction::{
    dry_run_default_fixture, execute_prediction, execute_prediction_from_match,
    hide_run_from_history, inspect_match_prediction_readiness, list_recent_runs, preview_route,
    read_run, PredictionAccess,
};
use crate::{
    ApplicationResult, PredictionCommand, PredictionExecution, RoutePreviewCommand,
    StoredMatchPredictionCommand,
};
use football_domain::{MatchPredictionReadiness, RouteDecision};
use football_model_api::ModelOutput;
use serde_json::Value;
use uuid::Uuid;

pub(crate) struct PredictionService;

impl PredictionService {
    pub(crate) fn new() -> Self {
        Self
    }

    pub(crate) async fn execute_prediction<P: PredictionAccess + ?Sized>(
        &self,
        port: &P,
        registry: &ModelRegistry,
        command: PredictionCommand,
    ) -> ApplicationResult<PredictionExecution> {
        execute_prediction::execute(port, registry, command).await
    }

    pub(crate) async fn inspect_match_prediction_readiness<P: PredictionAccess + ?Sized>(
        &self,
        port: &P,
        registry: &ModelRegistry,
        command: StoredMatchPredictionCommand,
    ) -> ApplicationResult<MatchPredictionReadiness> {
        inspect_match_prediction_readiness::execute(port, registry, command).await
    }

    pub(crate) async fn execute_prediction_from_match<P: PredictionAccess + ?Sized>(
        &self,
        port: &P,
        registry: &ModelRegistry,
        command: StoredMatchPredictionCommand,
    ) -> ApplicationResult<PredictionExecution> {
        execute_prediction_from_match::execute_formal(port, registry, command).await
    }

    pub(crate) async fn execute_shadow_prediction_from_match<P: PredictionAccess + ?Sized>(
        &self,
        port: &P,
        registry: &ModelRegistry,
        command: StoredMatchPredictionCommand,
    ) -> ApplicationResult<PredictionExecution> {
        execute_prediction_from_match::execute_shadow(port, registry, command).await
    }

    pub(crate) async fn preview_route<P: PredictionAccess + ?Sized>(
        &self,
        port: &P,
        registry: &ModelRegistry,
        command: RoutePreviewCommand,
    ) -> ApplicationResult<RouteDecision> {
        preview_route::execute(port, registry, command).await
    }

    pub(crate) fn dry_run_default_fixture(
        &self,
        registry: &ModelRegistry,
    ) -> ApplicationResult<ModelOutput> {
        dry_run_default_fixture::execute(registry)
    }

    pub(crate) async fn list_recent_runs<P: PredictionAccess + ?Sized>(
        &self,
        port: &P,
        limit: i64,
    ) -> ApplicationResult<Vec<ModelRunHistoryItem>> {
        list_recent_runs::execute(port, limit).await
    }

    pub(crate) async fn hide_run_from_history<P: PredictionAccess + ?Sized>(
        &self,
        port: &P,
        run_id: Uuid,
        reason: Option<String>,
    ) -> ApplicationResult<()> {
        hide_run_from_history::execute(port, run_id, reason).await
    }

    pub(crate) async fn read_run<P: PredictionAccess + ?Sized>(
        &self,
        port: &P,
        run_id: Uuid,
    ) -> ApplicationResult<Value> {
        read_run::execute(port, run_id).await
    }
}
