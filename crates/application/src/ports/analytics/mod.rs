use crate::ports::PortResult;
use async_trait::async_trait;
use football_domain::{
    AiAnalysisPackageData, AiAnalysisSuggestionRecord, AnalyticsOverview, AnalyticsRefreshRequest,
    BackgroundJob, DataQualityDecisionDraft, DataQualityFinding, EnqueueJobDraft,
    ParameterLifecycleReadiness, ParameterLifecycleReadinessRequest, ParameterPromotionRequest,
    ParameterRollbackRequest, ParameterShadowValidationRecord, ParameterShadowValidationRequest,
    ParameterTuningCandidateRecord,
};
use uuid::Uuid;

#[async_trait]
pub trait AnalyticsPort: Send + Sync {
    async fn overview(&self) -> PortResult<AnalyticsOverview>;
    async fn refresh(&self, request: &AnalyticsRefreshRequest) -> PortResult<AnalyticsOverview>;
    async fn run_data_quality_scan(&self) -> PortResult<Vec<DataQualityFinding>>;
    async fn decide_data_quality(
        &self,
        draft: &DataQualityDecisionDraft,
    ) -> PortResult<DataQualityFinding>;
    async fn build_ai_analysis_data(&self) -> PortResult<AiAnalysisPackageData>;
    async fn list_ai_suggestions(&self, limit: i64) -> PortResult<Vec<AiAnalysisSuggestionRecord>>;
}

#[async_trait]
pub trait JobQueuePort: Send + Sync {
    async fn enqueue(&self, draft: &EnqueueJobDraft) -> PortResult<BackgroundJob>;
    async fn list_jobs(&self, limit: i64) -> PortResult<Vec<BackgroundJob>>;
    async fn request_cancellation(&self, job_id: Uuid) -> PortResult<BackgroundJob>;
    async fn retry(&self, job_id: Uuid) -> PortResult<BackgroundJob>;
}

#[async_trait]
pub trait ParameterLifecyclePort: Send + Sync {
    async fn readiness(
        &self,
        request: &ParameterLifecycleReadinessRequest,
    ) -> PortResult<ParameterLifecycleReadiness>;
    async fn list_tuning_candidates(
        &self,
        limit: i64,
    ) -> PortResult<Vec<ParameterTuningCandidateRecord>>;
    async fn save_shadow_validation(
        &self,
        request: &ParameterShadowValidationRequest,
    ) -> PortResult<ParameterShadowValidationRecord>;
    async fn promote(&self, request: &ParameterPromotionRequest) -> PortResult<()>;
    async fn rollback(&self, request: &ParameterRollbackRequest) -> PortResult<()>;
}
