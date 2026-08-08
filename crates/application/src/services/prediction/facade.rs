use crate::composition::{model_run_list_item_from_port, ActiveDatabase};
use crate::{
    ApplicationError, ApplicationResult, ApplicationService, ModelRunListItem, PredictionCommand,
    PredictionExecution, RoutePreviewCommand, StoredMatchPredictionCommand,
};
use football_domain::{
    MatchPredictionReadiness, P4FreezeReadiness, P4FreezeTaskEventRecord, P4FreezeTaskRecord,
    P4MatchWorkspace, P4TaskWorkspace, PlanP4HorizonsCommand, RouteDecision,
};
use football_model_api::ModelOutput;
use serde_json::Value;
use uuid::Uuid;

impl ApplicationService {
    async fn prediction_session(&self) -> ApplicationResult<ActiveDatabase> {
        self.database
            .active_session()
            .await
            .ok_or(ApplicationError::DatabaseNotConnected)
    }

    pub async fn execute_prediction(
        &self,
        command: PredictionCommand,
    ) -> ApplicationResult<PredictionExecution> {
        let session = self.prediction_session().await?;
        self.prediction
            .execute_prediction(&session, &self.registry, command)
            .await
    }

    pub async fn inspect_match_prediction_readiness(
        &self,
        command: StoredMatchPredictionCommand,
    ) -> ApplicationResult<MatchPredictionReadiness> {
        let session = self.prediction_session().await?;
        self.prediction
            .inspect_match_prediction_readiness(&session, &self.registry, command)
            .await
    }

    pub async fn execute_prediction_from_match(
        &self,
        command: StoredMatchPredictionCommand,
    ) -> ApplicationResult<PredictionExecution> {
        let session = self.prediction_session().await?;
        self.prediction
            .execute_prediction_from_match(&session, &self.registry, command)
            .await
    }

    pub async fn execute_shadow_prediction_from_match(
        &self,
        command: StoredMatchPredictionCommand,
    ) -> ApplicationResult<PredictionExecution> {
        let session = self.prediction_session().await?;
        self.prediction
            .execute_shadow_prediction_from_match(&session, &self.registry, command)
            .await
    }

    pub async fn preview_route(
        &self,
        command: RoutePreviewCommand,
    ) -> ApplicationResult<RouteDecision> {
        let session = self.prediction_session().await?;
        self.prediction
            .preview_route(&session, &self.registry, command)
            .await
    }

    pub fn dry_run_default_fixture(&self) -> ApplicationResult<ModelOutput> {
        self.prediction.dry_run_default_fixture(&self.registry)
    }

    pub async fn list_recent_runs(&self, limit: i64) -> ApplicationResult<Vec<ModelRunListItem>> {
        let session = self.prediction_session().await?;
        let items = self.prediction.list_recent_runs(&session, limit).await?;
        items
            .into_iter()
            .map(model_run_list_item_from_port)
            .collect::<Result<Vec<_>, _>>()
            .map_err(Into::into)
    }

    pub async fn hide_run_from_history(
        &self,
        run_id: Uuid,
        reason: Option<String>,
    ) -> ApplicationResult<()> {
        let session = self.prediction_session().await?;
        self.prediction
            .hide_run_from_history(&session, run_id, reason)
            .await
    }

    pub async fn read_run(&self, run_id: Uuid) -> ApplicationResult<Value> {
        let session = self.prediction_session().await?;
        self.prediction.read_run(&session, run_id).await
    }

    pub async fn plan_p4_horizons(
        &self,
        command: PlanP4HorizonsCommand,
    ) -> ApplicationResult<Vec<P4FreezeTaskRecord>> {
        let session = self.prediction_session().await?;
        self.prediction.plan_p4_horizons(&session, command).await
    }

    pub async fn list_p4_freeze_tasks(
        &self,
        match_id: Option<Uuid>,
        limit: u32,
    ) -> ApplicationResult<Vec<P4FreezeTaskRecord>> {
        let session = self.prediction_session().await?;
        self.prediction
            .list_p4_freeze_tasks(&session, match_id, limit)
            .await
    }

    pub async fn read_p4_freeze_task(
        &self,
        task_id: Uuid,
    ) -> ApplicationResult<P4FreezeTaskRecord> {
        let session = self.prediction_session().await?;
        self.prediction.read_p4_freeze_task(&session, task_id).await
    }

    pub async fn list_p4_freeze_task_events(
        &self,
        task_id: Uuid,
    ) -> ApplicationResult<Vec<P4FreezeTaskEventRecord>> {
        let session = self.prediction_session().await?;
        self.prediction
            .list_p4_freeze_task_events(&session, task_id)
            .await
    }

    pub async fn p4_freeze_readiness(&self, task_id: Uuid) -> ApplicationResult<P4FreezeReadiness> {
        let session = self.prediction_session().await?;
        self.prediction.p4_freeze_readiness(&session, task_id).await
    }

    pub async fn read_p4_match_workspace(
        &self,
        match_id: Uuid,
    ) -> ApplicationResult<P4MatchWorkspace> {
        let session = self.prediction_session().await?;
        self.prediction
            .read_p4_match_workspace(&session, match_id)
            .await
    }

    pub async fn read_p4_task_workspace(
        &self,
        task_id: Uuid,
    ) -> ApplicationResult<P4TaskWorkspace> {
        let session = self.prediction_session().await?;
        self.prediction
            .read_p4_task_workspace(&session, task_id)
            .await
    }
}
