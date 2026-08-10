use crate::{ApplicationError, ApplicationResult, ApplicationService};
use football_domain::{
    AbilityCandidateDecisionDraft, AbilityCandidateStatus, AbilityUpdateCandidateRecord,
    MatchReviewDetail, MatchReviewDraft, MatchReviewPackageCommitRequest,
    MatchReviewPackageCommitResult, MatchReviewPackageConfirmationRequest,
    MatchReviewPackageFactsCommitResult, MatchReviewPackagePreview, MatchReviewPackageReviewResult,
    MatchReviewPackageSummary, MatchReviewPackageWorkflowRecord, MatchReviewSummary,
    ReviewableMatch,
};
use uuid::Uuid;

impl ApplicationService {
    async fn review_session(&self) -> ApplicationResult<crate::composition::ActiveDatabase> {
        self.database
            .active_session()
            .await
            .ok_or(ApplicationError::DatabaseNotConnected)
    }

    pub async fn generate_match_review(
        &self,
        draft: MatchReviewDraft,
    ) -> ApplicationResult<MatchReviewDetail> {
        let session = self.review_session().await?;
        self.review.generate_match_review(&session, draft).await
    }

    pub async fn list_reviewable_matches(
        &self,
        limit: u32,
    ) -> ApplicationResult<Vec<ReviewableMatch>> {
        let session = self.review_session().await?;
        self.review.list_reviewable_matches(&session, limit).await
    }

    pub async fn list_match_reviews(
        &self,
        limit: u32,
    ) -> ApplicationResult<Vec<MatchReviewSummary>> {
        let session = self.review_session().await?;
        self.review.list_match_reviews(&session, limit).await
    }

    pub async fn read_match_review(&self, review_id: Uuid) -> ApplicationResult<MatchReviewDetail> {
        let session = self.review_session().await?;
        self.review.read_match_review(&session, review_id).await
    }

    pub async fn list_ability_candidates(
        &self,
        status: Option<AbilityCandidateStatus>,
        limit: u32,
        match_review_id: Option<Uuid>,
    ) -> ApplicationResult<Vec<AbilityUpdateCandidateRecord>> {
        let session = self.review_session().await?;
        self.review
            .list_ability_candidates(&session, status, limit, match_review_id)
            .await
    }

    pub async fn decide_ability_candidate(
        &self,
        draft: AbilityCandidateDecisionDraft,
    ) -> ApplicationResult<AbilityUpdateCandidateRecord> {
        let session = self.review_session().await?;
        self.review.decide_ability_candidate(&session, draft).await
    }

    pub async fn export_match_review_package(
        &self,
        output_path: String,
        match_id: Uuid,
    ) -> ApplicationResult<MatchReviewPackageSummary> {
        let session = self.review_session().await?;
        self.review
            .export_match_review_package(&session, output_path, match_id)
            .await
    }

    pub async fn read_match_review_package_workflow(
        &self,
        match_id: Uuid,
    ) -> ApplicationResult<Option<MatchReviewPackageWorkflowRecord>> {
        let session = self.review_session().await?;
        self.review
            .read_match_review_package_workflow(&session, match_id)
            .await
    }

    pub async fn preview_match_review_package(
        &self,
        input_path: String,
        expected_match_id: Option<Uuid>,
    ) -> ApplicationResult<MatchReviewPackagePreview> {
        let session = self.review_session().await?;
        self.review
            .preview_match_review_package(&session, input_path, expected_match_id)
            .await
    }

    pub async fn confirm_match_review_package(
        &self,
        request: MatchReviewPackageConfirmationRequest,
    ) -> ApplicationResult<MatchReviewPackageWorkflowRecord> {
        let session = self.review_session().await?;
        self.review
            .confirm_match_review_package(&session, request)
            .await
    }

    pub async fn commit_match_review_package_facts(
        &self,
        package_id: Uuid,
    ) -> ApplicationResult<MatchReviewPackageFactsCommitResult> {
        let session = self.review_session().await?;
        self.review
            .commit_match_review_package_facts(&session, package_id)
            .await
    }

    pub async fn generate_match_review_from_package(
        &self,
        package_id: Uuid,
    ) -> ApplicationResult<MatchReviewPackageReviewResult> {
        let session = self.review_session().await?;
        self.review
            .generate_match_review_from_package(&session, package_id)
            .await
    }

    pub async fn commit_match_review_package(
        &self,
        request: MatchReviewPackageCommitRequest,
    ) -> ApplicationResult<MatchReviewPackageCommitResult> {
        let session = self.review_session().await?;
        self.review
            .commit_match_review_package(&session, request)
            .await
    }
}
