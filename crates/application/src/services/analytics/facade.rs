use crate::{ApplicationError, ApplicationResult, ApplicationService};
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

impl ApplicationService {
    async fn analytics_session(&self) -> ApplicationResult<crate::composition::DatabaseSession> {
        self.database
            .active_session()
            .await
            .ok_or(ApplicationError::DatabaseNotConnected)
    }

    pub async fn analytics_overview(&self) -> ApplicationResult<AnalyticsOverview> {
        let session = self.analytics_session().await?;
        self.analytics.overview(&session).await
    }

    pub async fn enqueue_analysis_job(
        &self,
        draft: EnqueueJobDraft,
    ) -> ApplicationResult<BackgroundJob> {
        let session = self.analytics_session().await?;
        self.analytics.enqueue_job(&session, draft).await
    }

    pub async fn list_background_jobs(&self, limit: u32) -> ApplicationResult<Vec<BackgroundJob>> {
        let session = self.analytics_session().await?;
        self.analytics.list_jobs(&session, limit).await
    }

    pub async fn cancel_background_job(&self, job_id: Uuid) -> ApplicationResult<BackgroundJob> {
        let session = self.analytics_session().await?;
        self.analytics.cancel_job(&session, job_id).await
    }

    pub async fn retry_background_job(&self, job_id: Uuid) -> ApplicationResult<BackgroundJob> {
        let session = self.analytics_session().await?;
        self.analytics.retry_job(&session, job_id).await
    }

    pub async fn decide_data_quality_finding(
        &self,
        draft: DataQualityDecisionDraft,
    ) -> ApplicationResult<DataQualityFinding> {
        let session = self.analytics_session().await?;
        self.analytics.decide_data_quality(&session, draft).await
    }

    pub fn export_ai_analysis_response_template(
        &self,
        output_path: String,
        source_package_id: Option<Uuid>,
    ) -> ApplicationResult<String> {
        self.analytics
            .export_ai_response_template(output_path, source_package_id)
    }

    pub async fn export_ai_analysis_package(
        &self,
        output_path: String,
    ) -> ApplicationResult<AiAnalysisPackageSummary> {
        let session = self.analytics_session().await?;
        self.analytics
            .export_ai_package(&session, output_path)
            .await
    }

    pub fn preview_ai_analysis_response(
        &self,
        input_path: String,
    ) -> ApplicationResult<AiAnalysisResponsePreview> {
        self.analytics.preview_ai_response(input_path)
    }

    pub async fn import_ai_analysis_response(
        &self,
        input_path: String,
    ) -> ApplicationResult<Vec<AiAnalysisSuggestionRecord>> {
        let session = self.analytics_session().await?;
        self.analytics
            .import_ai_response(&session, input_path)
            .await
    }

    pub async fn list_ai_analysis_suggestions(
        &self,
        status: Option<String>,
        limit: u32,
    ) -> ApplicationResult<Vec<AiAnalysisSuggestionRecord>> {
        let session = self.analytics_session().await?;
        self.analytics
            .list_ai_suggestions(&session, status, limit)
            .await
    }

    pub async fn decide_ai_analysis_suggestion(
        &self,
        draft: AiSuggestionDecisionDraft,
    ) -> ApplicationResult<AiAnalysisSuggestionRecord> {
        let session = self.analytics_session().await?;
        self.analytics.decide_ai_suggestion(&session, draft).await
    }

    pub async fn generate_parameter_tuning_candidate(
        &self,
        draft: ParameterTuningDraft,
    ) -> ApplicationResult<ParameterTuningCandidateRecord> {
        self.analytics.generate_parameter_candidate(draft).await
    }

    pub async fn list_parameter_tuning_candidates(
        &self,
        limit: u32,
    ) -> ApplicationResult<Vec<ParameterTuningCandidateRecord>> {
        let session = self.analytics_session().await?;
        self.analytics
            .list_parameter_candidates(&session, limit)
            .await
    }

    pub async fn decide_parameter_tuning_candidate(
        &self,
        draft: ParameterTuningDecisionDraft,
    ) -> ApplicationResult<ParameterTuningCandidateRecord> {
        let session = self.analytics_session().await?;
        self.analytics
            .decide_parameter_candidate(&session, draft)
            .await
    }

    pub async fn parameter_lifecycle_readiness(
        &self,
        request: ParameterLifecycleReadinessRequest,
    ) -> ApplicationResult<ParameterLifecycleReadiness> {
        let session = self.analytics_session().await?;
        self.analytics.parameter_readiness(&session, request).await
    }

    pub async fn run_parameter_shadow_validation(
        &self,
        request: ParameterShadowValidationRequest,
    ) -> ApplicationResult<ParameterShadowValidationRecord> {
        let session = self.analytics_session().await?;
        self.analytics
            .run_shadow_validation(&session, &self.registry, request)
            .await
    }

    pub async fn list_parameter_shadow_validations(
        &self,
        candidate_id: Uuid,
    ) -> ApplicationResult<Vec<ParameterShadowValidationRecord>> {
        let session = self.analytics_session().await?;
        self.analytics
            .list_shadow_validations(&session, candidate_id)
            .await
    }

    pub async fn promote_parameter_candidate(
        &self,
        request: ParameterPromotionRequest,
    ) -> ApplicationResult<ParameterPromotionDecisionRecord> {
        let session = self.analytics_session().await?;
        self.analytics.promote_parameter(&session, request).await
    }

    pub async fn rollback_parameter_candidate(
        &self,
        request: ParameterRollbackRequest,
    ) -> ApplicationResult<ParameterPromotionDecisionRecord> {
        let session = self.analytics_session().await?;
        self.analytics.rollback_parameter(&session, request).await
    }

    pub async fn list_parameter_promotion_decisions(
        &self,
        candidate_id: Uuid,
    ) -> ApplicationResult<Vec<ParameterPromotionDecisionRecord>> {
        let session = self.analytics_session().await?;
        self.analytics
            .list_promotion_decisions(&session, candidate_id)
            .await
    }
}
