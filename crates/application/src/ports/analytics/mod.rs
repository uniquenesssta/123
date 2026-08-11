mod contracts;

pub use contracts::{AnalyticsJobProgressPayload, AnalyticsJobResult, ParameterDefinition};

use crate::ports::PortResult;
use async_trait::async_trait;
use football_domain::{
    AiAnalysisPackageData, AiAnalysisPackageSummary, AiAnalysisResponsePreview,
    AiAnalysisSuggestionRecord, AiSuggestionDecisionDraft, AnalyticsOverview,
    AnalyticsRefreshRequest, BackgroundJob, DataQualityDecisionDraft, DataQualityFinding,
    DataQualitySummary, EnqueueJobDraft, ParameterLifecycleReadiness,
    ParameterLifecycleReadinessRequest, ParameterPromotionDecisionRecord,
    ParameterPromotionRequest, ParameterReplayFixture, ParameterRollbackRequest,
    ParameterShadowValidationRecord, ParameterTuningCandidateRecord, ParameterTuningDecisionDraft,
    QueryPerformanceSummary,
};
use uuid::Uuid;

#[async_trait]
pub trait AnalyticsPort: Send + Sync {
    async fn overview(&self) -> PortResult<AnalyticsOverview>;
    async fn refresh(&self, request: &AnalyticsRefreshRequest) -> PortResult<AnalyticsOverview>;
    async fn run_data_quality_scan(&self) -> PortResult<DataQualitySummary>;
    async fn decide_data_quality(
        &self,
        draft: &DataQualityDecisionDraft,
    ) -> PortResult<DataQualityFinding>;
    async fn capture_query_performance(&self) -> PortResult<QueryPerformanceSummary>;
    async fn build_ai_analysis_data(&self) -> PortResult<AiAnalysisPackageData>;
    async fn record_ai_export(&self, summary: &AiAnalysisPackageSummary) -> PortResult<()>;
    async fn import_ai_response(
        &self,
        input_path: &str,
        preview: &AiAnalysisResponsePreview,
    ) -> PortResult<Vec<AiAnalysisSuggestionRecord>>;
    async fn list_ai_suggestions(
        &self,
        status: Option<&str>,
        limit: u32,
    ) -> PortResult<Vec<AiAnalysisSuggestionRecord>>;
    async fn decide_ai_suggestion(
        &self,
        draft: &AiSuggestionDecisionDraft,
    ) -> PortResult<AiAnalysisSuggestionRecord>;
}

#[async_trait]
pub trait JobQueuePort: Send + Sync {
    async fn enqueue(&self, draft: &EnqueueJobDraft) -> PortResult<BackgroundJob>;
    async fn list_jobs(&self, limit: u32) -> PortResult<Vec<BackgroundJob>>;
    async fn request_cancellation(&self, job_id: Uuid) -> PortResult<BackgroundJob>;
    async fn retry(&self, job_id: Uuid) -> PortResult<BackgroundJob>;
    async fn claim_next_by_types(&self, job_types: &[&str]) -> PortResult<Option<BackgroundJob>>;
    async fn update_progress(
        &self,
        job_id: Uuid,
        progress: f64,
        message: &str,
        payload: AnalyticsJobProgressPayload,
    ) -> PortResult<bool>;
    async fn complete(&self, job_id: Uuid, result: AnalyticsJobResult) -> PortResult<()>;
    async fn fail(&self, job_id: Uuid, error_message: &str) -> PortResult<()>;
}

#[async_trait]
pub trait ParameterLifecyclePort: Send + Sync {
    async fn readiness(
        &self,
        request: &ParameterLifecycleReadinessRequest,
    ) -> PortResult<ParameterLifecycleReadiness>;
    async fn list_tuning_candidates(
        &self,
        limit: u32,
    ) -> PortResult<Vec<ParameterTuningCandidateRecord>>;
    async fn read_tuning_candidate(
        &self,
        candidate_id: Uuid,
    ) -> PortResult<ParameterTuningCandidateRecord>;
    async fn decide_tuning_candidate(
        &self,
        draft: &ParameterTuningDecisionDraft,
    ) -> PortResult<ParameterTuningCandidateRecord>;
    async fn load_replay_fixtures(
        &self,
        competition_id: Uuid,
        competition_profile_id: Uuid,
        snapshot_type: &str,
        baseline_model_version_id: Uuid,
        baseline_parameter_set_id: Uuid,
    ) -> PortResult<Vec<ParameterReplayFixture>>;
    async fn read_parameter_set_definition(
        &self,
        parameter_set_id: Uuid,
    ) -> PortResult<ParameterDefinition>;
    async fn save_shadow_validation(
        &self,
        record: &ParameterShadowValidationRecord,
    ) -> PortResult<ParameterShadowValidationRecord>;
    async fn list_shadow_validations(
        &self,
        candidate_id: Uuid,
    ) -> PortResult<Vec<ParameterShadowValidationRecord>>;
    async fn promote(
        &self,
        request: &ParameterPromotionRequest,
    ) -> PortResult<ParameterPromotionDecisionRecord>;
    async fn rollback(
        &self,
        request: &ParameterRollbackRequest,
    ) -> PortResult<ParameterPromotionDecisionRecord>;
    async fn list_promotion_decisions(
        &self,
        candidate_id: Uuid,
    ) -> PortResult<Vec<ParameterPromotionDecisionRecord>>;
}
