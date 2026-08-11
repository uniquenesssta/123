use super::super::port_registry::{map_persistence_error, ActiveDatabase};
use crate::ports::{
    analytics::{AnalyticsPort, ParameterDefinition, ParameterLifecyclePort},
    PortResult,
};
use async_trait::async_trait;
use football_domain::{
    AiAnalysisPackageData, AiAnalysisPackageSummary, AiAnalysisResponsePreview,
    AiAnalysisSuggestionRecord, AiSuggestionDecisionDraft, AnalyticsOverview,
    AnalyticsRefreshRequest, DataQualityDecisionDraft, DataQualityFinding, DataQualitySummary,
    ParameterLifecycleReadiness, ParameterLifecycleReadinessRequest,
    ParameterPromotionDecisionRecord, ParameterPromotionRequest, ParameterReplayFixture,
    ParameterRollbackRequest, ParameterShadowValidationRecord, ParameterTuningCandidateRecord,
    ParameterTuningDecisionDraft, QueryPerformanceSummary,
};
use uuid::Uuid;

#[async_trait]
impl AnalyticsPort for ActiveDatabase {
    async fn overview(&self) -> PortResult<AnalyticsOverview> {
        self.transition_store()
            .analytics_overview()
            .await
            .map_err(map_persistence_error)
    }

    async fn refresh(&self, request: &AnalyticsRefreshRequest) -> PortResult<AnalyticsOverview> {
        self.transition_store()
            .refresh_analytics(request)
            .await
            .map_err(map_persistence_error)
    }

    async fn run_data_quality_scan(&self) -> PortResult<DataQualitySummary> {
        self.transition_store()
            .run_data_quality_scan()
            .await
            .map_err(map_persistence_error)
    }

    async fn decide_data_quality(
        &self,
        draft: &DataQualityDecisionDraft,
    ) -> PortResult<DataQualityFinding> {
        self.transition_store()
            .decide_data_quality_finding(draft)
            .await
            .map_err(map_persistence_error)
    }

    async fn capture_query_performance(&self) -> PortResult<QueryPerformanceSummary> {
        self.transition_store()
            .capture_query_performance()
            .await
            .map_err(map_persistence_error)
    }

    async fn build_ai_analysis_data(&self) -> PortResult<AiAnalysisPackageData> {
        self.transition_store()
            .build_ai_analysis_data()
            .await
            .map_err(map_persistence_error)
    }

    async fn record_ai_export(&self, summary: &AiAnalysisPackageSummary) -> PortResult<()> {
        self.transition_store()
            .record_ai_export(summary)
            .await
            .map_err(map_persistence_error)
    }

    async fn import_ai_response(
        &self,
        input_path: &str,
        preview: &AiAnalysisResponsePreview,
    ) -> PortResult<Vec<AiAnalysisSuggestionRecord>> {
        self.transition_store()
            .import_ai_response(input_path, preview)
            .await
            .map_err(map_persistence_error)
    }

    async fn list_ai_suggestions(
        &self,
        status: Option<&str>,
        limit: u32,
    ) -> PortResult<Vec<AiAnalysisSuggestionRecord>> {
        self.transition_store()
            .list_ai_suggestions(status, limit)
            .await
            .map_err(map_persistence_error)
    }

    async fn decide_ai_suggestion(
        &self,
        draft: &AiSuggestionDecisionDraft,
    ) -> PortResult<AiAnalysisSuggestionRecord> {
        self.transition_store()
            .decide_ai_suggestion(draft)
            .await
            .map_err(map_persistence_error)
    }
}

#[async_trait]
impl ParameterLifecyclePort for ActiveDatabase {
    async fn readiness(
        &self,
        request: &ParameterLifecycleReadinessRequest,
    ) -> PortResult<ParameterLifecycleReadiness> {
        self.transition_store()
            .parameter_lifecycle_readiness(request)
            .await
            .map_err(map_persistence_error)
    }

    async fn list_tuning_candidates(
        &self,
        limit: u32,
    ) -> PortResult<Vec<ParameterTuningCandidateRecord>> {
        self.transition_store()
            .list_parameter_tuning_candidates(limit)
            .await
            .map_err(map_persistence_error)
    }

    async fn read_tuning_candidate(
        &self,
        candidate_id: Uuid,
    ) -> PortResult<ParameterTuningCandidateRecord> {
        self.transition_store()
            .read_parameter_tuning_candidate(candidate_id)
            .await
            .map_err(map_persistence_error)
    }

    async fn decide_tuning_candidate(
        &self,
        draft: &ParameterTuningDecisionDraft,
    ) -> PortResult<ParameterTuningCandidateRecord> {
        self.transition_store()
            .decide_parameter_tuning_candidate(draft)
            .await
            .map_err(map_persistence_error)
    }

    async fn load_replay_fixtures(
        &self,
        competition_id: Uuid,
        competition_profile_id: Uuid,
        snapshot_type: &str,
        baseline_model_version_id: Uuid,
        baseline_parameter_set_id: Uuid,
    ) -> PortResult<Vec<ParameterReplayFixture>> {
        self.transition_store()
            .load_parameter_replay_fixtures(
                competition_id,
                competition_profile_id,
                snapshot_type,
                baseline_model_version_id,
                baseline_parameter_set_id,
            )
            .await
            .map_err(map_persistence_error)
    }

    async fn read_parameter_set_definition(
        &self,
        parameter_set_id: Uuid,
    ) -> PortResult<ParameterDefinition> {
        let value = self
            .transition_store()
            .read_parameter_set_definition(parameter_set_id)
            .await
            .map_err(map_persistence_error)?;
        Ok(ParameterDefinition::from_value(value))
    }

    async fn save_shadow_validation(
        &self,
        record: &ParameterShadowValidationRecord,
    ) -> PortResult<ParameterShadowValidationRecord> {
        self.transition_store()
            .save_parameter_shadow_validation(record)
            .await
            .map_err(map_persistence_error)
    }

    async fn list_shadow_validations(
        &self,
        candidate_id: Uuid,
    ) -> PortResult<Vec<ParameterShadowValidationRecord>> {
        self.transition_store()
            .list_parameter_shadow_validations(candidate_id)
            .await
            .map_err(map_persistence_error)
    }

    async fn promote(
        &self,
        request: &ParameterPromotionRequest,
    ) -> PortResult<ParameterPromotionDecisionRecord> {
        self.transition_store()
            .promote_parameter_candidate(request)
            .await
            .map_err(map_persistence_error)
    }

    async fn rollback(
        &self,
        request: &ParameterRollbackRequest,
    ) -> PortResult<ParameterPromotionDecisionRecord> {
        self.transition_store()
            .rollback_parameter_candidate(request)
            .await
            .map_err(map_persistence_error)
    }

    async fn list_promotion_decisions(
        &self,
        candidate_id: Uuid,
    ) -> PortResult<Vec<ParameterPromotionDecisionRecord>> {
        self.transition_store()
            .list_parameter_promotion_decisions(candidate_id)
            .await
            .map_err(map_persistence_error)
    }
}
