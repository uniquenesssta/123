use crate::model_registry::ModelRegistry;
use crate::ports::prediction::ModelRunHistoryItem;
use crate::use_cases::prediction::{
    dry_run_default_fixture, execute_prediction, execute_prediction_from_match,
    hide_run_from_history, inspect_match_prediction_readiness, list_p4_freeze_task_events,
    list_p4_freeze_tasks, list_recent_runs, p4_freeze_readiness, plan_p4_horizons, preview_route,
    read_p4_freeze_task, read_p4_match_workspace, read_p4_task_workspace, read_run,
    P4PlanningAccess, PredictionAccess,
};
use crate::{
    ApplicationResult, PredictionCommand, PredictionExecution, RoutePreviewCommand,
    StoredMatchPredictionCommand,
};
use football_domain::{
    MatchPredictionReadiness, P4FreezeReadiness, P4FreezeTaskEventRecord, P4FreezeTaskRecord,
    P4MatchWorkspace, P4TaskWorkspace, PlanP4HorizonsCommand, RouteDecision,
};
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

    pub(crate) async fn plan_p4_horizons<P: P4PlanningAccess + ?Sized>(
        &self,
        port: &P,
        command: PlanP4HorizonsCommand,
    ) -> ApplicationResult<Vec<P4FreezeTaskRecord>> {
        plan_p4_horizons::execute(port, command).await
    }

    pub(crate) async fn list_p4_freeze_tasks<
        P: crate::ports::prediction::PredictionWorkflowPort + ?Sized,
    >(
        &self,
        port: &P,
        match_id: Option<Uuid>,
        limit: u32,
    ) -> ApplicationResult<Vec<P4FreezeTaskRecord>> {
        list_p4_freeze_tasks::execute(port, match_id, limit).await
    }

    pub(crate) async fn read_p4_freeze_task<
        P: crate::ports::prediction::PredictionWorkflowPort + ?Sized,
    >(
        &self,
        port: &P,
        task_id: Uuid,
    ) -> ApplicationResult<P4FreezeTaskRecord> {
        read_p4_freeze_task::execute(port, task_id).await
    }

    pub(crate) async fn list_p4_freeze_task_events<
        P: crate::ports::prediction::PredictionWorkflowPort + ?Sized,
    >(
        &self,
        port: &P,
        task_id: Uuid,
    ) -> ApplicationResult<Vec<P4FreezeTaskEventRecord>> {
        list_p4_freeze_task_events::execute(port, task_id).await
    }

    pub(crate) async fn p4_freeze_readiness<
        P: crate::ports::prediction::PredictionWorkflowPort + ?Sized,
    >(
        &self,
        port: &P,
        task_id: Uuid,
    ) -> ApplicationResult<P4FreezeReadiness> {
        p4_freeze_readiness::execute(port, task_id).await
    }

    pub(crate) async fn read_p4_match_workspace<
        P: crate::ports::prediction::PredictionWorkflowPort + ?Sized,
    >(
        &self,
        port: &P,
        match_id: Uuid,
    ) -> ApplicationResult<P4MatchWorkspace> {
        read_p4_match_workspace::execute(port, match_id).await
    }

    pub(crate) async fn read_p4_task_workspace<
        P: crate::ports::prediction::PredictionWorkflowPort + ?Sized,
    >(
        &self,
        port: &P,
        task_id: Uuid,
    ) -> ApplicationResult<P4TaskWorkspace> {
        read_p4_task_workspace::execute(port, task_id).await
    }

    pub(crate) async fn read_run<P: PredictionAccess + ?Sized>(
        &self,
        port: &P,
        run_id: Uuid,
    ) -> ApplicationResult<Value> {
        read_run::execute(port, run_id).await
    }
}
