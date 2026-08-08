use super::super::port_registry::{map_persistence_error, ActiveDatabase, ModelRunListItem};
use crate::ports::{
    prediction::{ModelRunHistoryItem, ModelRunPort, PredictionInputPort, SerializedModelRun},
    PortError, PortErrorKind, PortResult,
};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use football_domain::{PredictionSummary, PreparedMatchPredictionInput, RouteDecision};
use football_model_api::{ModelOutput, ModelRequest};
use uuid::Uuid;

#[async_trait]
impl PredictionInputPort for ActiveDatabase {
    async fn prepare_match_input(
        &self,
        match_id: Uuid,
        snapshot_type: &str,
        model_family: &str,
    ) -> PortResult<PreparedMatchPredictionInput> {
        self.transition_store()
            .prepare_match_prediction_input(match_id, snapshot_type, model_family)
            .await
            .map_err(map_persistence_error)
    }

    async fn prepare_match_input_at(
        &self,
        match_id: Uuid,
        snapshot_type: &str,
        model_family: &str,
        reference_time: DateTime<Utc>,
    ) -> PortResult<PreparedMatchPredictionInput> {
        self.transition_store()
            .prepare_match_prediction_input_at(
                match_id,
                snapshot_type,
                model_family,
                reference_time,
            )
            .await
            .map_err(map_persistence_error)
    }
}

#[async_trait]
impl ModelRunPort for ActiveDatabase {
    async fn save_successful_run(
        &self,
        decision: &RouteDecision,
        request: &ModelRequest,
        output: &ModelOutput,
        duration_ms: i64,
    ) -> PortResult<Uuid> {
        self.transition_store()
            .save_successful_run(decision, request, output, duration_ms)
            .await
            .map_err(map_persistence_error)
    }

    async fn hide_run_from_history(&self, run_id: Uuid, reason: Option<&str>) -> PortResult<()> {
        self.transition_store()
            .hide_run_from_history(run_id, reason)
            .await
            .map_err(map_persistence_error)
    }

    async fn list_recent_runs(&self, limit: i64) -> PortResult<Vec<ModelRunHistoryItem>> {
        self.transition_store()
            .list_recent_runs(limit)
            .await
            .map_err(map_persistence_error)?
            .into_iter()
            .map(|item| {
                let summary: PredictionSummary =
                    serde_json::from_value(item.summary).map_err(|error| {
                        PortError::new(PortErrorKind::Serialization, error.to_string())
                    })?;
                Ok(ModelRunHistoryItem {
                    id: item.id,
                    match_key: item.match_key,
                    competition_name: item.competition_name,
                    home_team_name: item.home_team_name,
                    away_team_name: item.away_team_name,
                    kickoff_time: item.kickoff_time,
                    snapshot_type: item.snapshot_type,
                    model_key: item.model_key,
                    model_version: item.model_version,
                    parameter_version: item.parameter_version,
                    rule_package_name: item.rule_package_name,
                    summary,
                    top_scoreline: item.top_scoreline,
                    top_scoreline_probability: item.top_scoreline_probability,
                    created_at: item.created_at,
                    completed_at: item.completed_at,
                    duration_ms: item.duration_ms,
                    input_readiness_level: item.input_readiness_level,
                    input_readiness_score: item.input_readiness_score,
                    input_manifest_sha256: item.input_manifest_sha256,
                })
            })
            .collect()
    }

    async fn read_run_document(&self, run_id: Uuid) -> PortResult<SerializedModelRun> {
        let value = self
            .transition_store()
            .read_run(run_id)
            .await
            .map_err(map_persistence_error)?;
        let json = serde_json::to_string(&value)
            .map_err(|error| PortError::new(PortErrorKind::Serialization, error.to_string()))?;
        Ok(SerializedModelRun { json })
    }
}

pub(crate) fn model_run_list_item_from_port(
    item: ModelRunHistoryItem,
) -> Result<ModelRunListItem, serde_json::Error> {
    Ok(ModelRunListItem {
        id: item.id,
        match_key: item.match_key,
        competition_name: item.competition_name,
        home_team_name: item.home_team_name,
        away_team_name: item.away_team_name,
        kickoff_time: item.kickoff_time,
        snapshot_type: item.snapshot_type,
        model_key: item.model_key,
        model_version: item.model_version,
        parameter_version: item.parameter_version,
        rule_package_name: item.rule_package_name,
        summary: serde_json::to_value(item.summary)?,
        top_scoreline: item.top_scoreline,
        top_scoreline_probability: item.top_scoreline_probability,
        created_at: item.created_at,
        completed_at: item.completed_at,
        duration_ms: item.duration_ms,
        input_readiness_level: item.input_readiness_level,
        input_readiness_score: item.input_readiness_score,
        input_manifest_sha256: item.input_manifest_sha256,
    })
}
