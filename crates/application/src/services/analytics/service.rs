use crate::{
    model_registry::ModelRegistry,
    ports::analytics::{AnalyticsPort, JobQueuePort, ParameterLifecyclePort},
    use_cases::analytics,
    ApplicationResult,
};
use football_domain::{
    AiAnalysisPackageSummary, AiAnalysisResponsePreview, AiAnalysisSuggestionRecord,
    AiSuggestionDecisionDraft, AnalyticsOverview, BackgroundJob, DataQualityDecisionDraft,
    DataQualityFinding, EnqueueJobDraft, ParameterLifecycleReadiness,
    ParameterLifecycleReadinessRequest, ParameterPromotionDecisionRecord,
    ParameterPromotionRequest, ParameterRollbackRequest, ParameterShadowValidationRecord,
    ParameterShadowValidationRequest, ParameterTuningCandidateRecord, ParameterTuningDecisionDraft,
    ParameterTuningDraft,
};
use uuid::Uuid;

pub(crate) struct AnalyticsService;

impl AnalyticsService {
    pub(crate) fn new() -> Self {
        Self
    }

    pub(crate) fn start_job_worker<P>(&self, port: P)
    where
        P: AnalyticsPort + JobQueuePort + Clone + Send + Sync + 'static,
    {
        analytics::jobs::worker::spawn(port);
    }

    pub(crate) async fn overview<P>(&self, port: &P) -> ApplicationResult<AnalyticsOverview>
    where
        P: AnalyticsPort + ?Sized,
    {
        analytics::analytics_overview::execute(port).await
    }

    pub(crate) async fn enqueue_job<P>(
        &self,
        port: &P,
        draft: EnqueueJobDraft,
    ) -> ApplicationResult<BackgroundJob>
    where
        P: AnalyticsPort + JobQueuePort + Clone + Send + Sync + 'static,
    {
        analytics::jobs::enqueue_analysis_job::execute(port, draft).await
    }

    pub(crate) async fn list_jobs<P>(
        &self,
        port: &P,
        limit: u32,
    ) -> ApplicationResult<Vec<BackgroundJob>>
    where
        P: JobQueuePort + ?Sized,
    {
        analytics::jobs::list_background_jobs::execute(port, limit).await
    }

    pub(crate) async fn cancel_job<P>(
        &self,
        port: &P,
        job_id: Uuid,
    ) -> ApplicationResult<BackgroundJob>
    where
        P: JobQueuePort + ?Sized,
    {
        analytics::jobs::cancel_background_job::execute(port, job_id).await
    }

    pub(crate) async fn retry_job<P>(
        &self,
        port: &P,
        job_id: Uuid,
    ) -> ApplicationResult<BackgroundJob>
    where
        P: AnalyticsPort + JobQueuePort + Clone + Send + Sync + 'static,
    {
        analytics::jobs::retry_background_job::execute(port, job_id).await
    }

    pub(crate) async fn decide_data_quality<P>(
        &self,
        port: &P,
        draft: DataQualityDecisionDraft,
    ) -> ApplicationResult<DataQualityFinding>
    where
        P: AnalyticsPort + ?Sized,
    {
        analytics::decide_data_quality_finding::execute(port, draft).await
    }

    pub(crate) fn export_ai_response_template(
        &self,
        output_path: String,
        source_package_id: Option<Uuid>,
    ) -> ApplicationResult<String> {
        analytics::ai_package::export_ai_analysis_response_template::execute(
            output_path,
            source_package_id,
        )
    }

    pub(crate) async fn export_ai_package<P>(
        &self,
        port: &P,
        output_path: String,
    ) -> ApplicationResult<AiAnalysisPackageSummary>
    where
        P: AnalyticsPort + ?Sized,
    {
        analytics::ai_package::export_ai_analysis_package::execute(port, output_path).await
    }

    pub(crate) fn preview_ai_response(
        &self,
        input_path: String,
    ) -> ApplicationResult<AiAnalysisResponsePreview> {
        analytics::ai_package::preview_ai_analysis_response::execute(input_path)
    }

    pub(crate) async fn import_ai_response<P>(
        &self,
        port: &P,
        input_path: String,
    ) -> ApplicationResult<Vec<AiAnalysisSuggestionRecord>>
    where
        P: AnalyticsPort + ?Sized,
    {
        analytics::ai_package::import_ai_analysis_response::execute(port, input_path).await
    }

    pub(crate) async fn list_ai_suggestions<P>(
        &self,
        port: &P,
        status: Option<String>,
        limit: u32,
    ) -> ApplicationResult<Vec<AiAnalysisSuggestionRecord>>
    where
        P: AnalyticsPort + ?Sized,
    {
        analytics::ai_package::list_ai_analysis_suggestions::execute(port, status, limit).await
    }

    pub(crate) async fn decide_ai_suggestion<P>(
        &self,
        port: &P,
        draft: AiSuggestionDecisionDraft,
    ) -> ApplicationResult<AiAnalysisSuggestionRecord>
    where
        P: AnalyticsPort + ?Sized,
    {
        analytics::ai_package::decide_ai_analysis_suggestion::execute(port, draft).await
    }

    pub(crate) async fn generate_parameter_candidate(
        &self,
        draft: ParameterTuningDraft,
    ) -> ApplicationResult<ParameterTuningCandidateRecord> {
        analytics::parameter_lifecycle::generate_parameter_tuning_candidate::execute(draft).await
    }

    pub(crate) async fn list_parameter_candidates<P>(
        &self,
        port: &P,
        limit: u32,
    ) -> ApplicationResult<Vec<ParameterTuningCandidateRecord>>
    where
        P: ParameterLifecyclePort + ?Sized,
    {
        analytics::parameter_lifecycle::list_parameter_tuning_candidates::execute(port, limit).await
    }

    pub(crate) async fn decide_parameter_candidate<P>(
        &self,
        port: &P,
        draft: ParameterTuningDecisionDraft,
    ) -> ApplicationResult<ParameterTuningCandidateRecord>
    where
        P: ParameterLifecyclePort + ?Sized,
    {
        analytics::parameter_lifecycle::decide_parameter_tuning_candidate::execute(port, draft)
            .await
    }

    pub(crate) async fn parameter_readiness<P>(
        &self,
        port: &P,
        request: ParameterLifecycleReadinessRequest,
    ) -> ApplicationResult<ParameterLifecycleReadiness>
    where
        P: ParameterLifecyclePort + ?Sized,
    {
        analytics::parameter_lifecycle::parameter_lifecycle_readiness::execute(port, request).await
    }

    pub(crate) async fn run_shadow_validation<P>(
        &self,
        port: &P,
        registry: &ModelRegistry,
        request: ParameterShadowValidationRequest,
    ) -> ApplicationResult<ParameterShadowValidationRecord>
    where
        P: ParameterLifecyclePort + ?Sized,
    {
        analytics::parameter_lifecycle::run_parameter_shadow_validation::execute(
            port, registry, request,
        )
        .await
    }

    pub(crate) async fn list_shadow_validations<P>(
        &self,
        port: &P,
        candidate_id: Uuid,
    ) -> ApplicationResult<Vec<ParameterShadowValidationRecord>>
    where
        P: ParameterLifecyclePort + ?Sized,
    {
        analytics::parameter_lifecycle::list_parameter_shadow_validations::execute(
            port,
            candidate_id,
        )
        .await
    }

    pub(crate) async fn promote_parameter<P>(
        &self,
        port: &P,
        request: ParameterPromotionRequest,
    ) -> ApplicationResult<ParameterPromotionDecisionRecord>
    where
        P: ParameterLifecyclePort + ?Sized,
    {
        analytics::parameter_lifecycle::promote_parameter_candidate::execute(port, request).await
    }

    pub(crate) async fn rollback_parameter<P>(
        &self,
        port: &P,
        request: ParameterRollbackRequest,
    ) -> ApplicationResult<ParameterPromotionDecisionRecord>
    where
        P: ParameterLifecyclePort + ?Sized,
    {
        analytics::parameter_lifecycle::rollback_parameter_candidate::execute(port, request).await
    }

    pub(crate) async fn list_promotion_decisions<P>(
        &self,
        port: &P,
        candidate_id: Uuid,
    ) -> ApplicationResult<Vec<ParameterPromotionDecisionRecord>>
    where
        P: ParameterLifecyclePort + ?Sized,
    {
        analytics::parameter_lifecycle::list_parameter_promotion_decisions::execute(
            port,
            candidate_id,
        )
        .await
    }
}
