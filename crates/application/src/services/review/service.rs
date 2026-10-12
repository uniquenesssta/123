use crate::{
    ports::{
        lineup::LineupPort,
        review::{
            MatchReviewPackageFactsPort, MatchReviewPackageSourcePort, MatchReviewPackageStatePort,
            MatchReviewPort,
        },
    },
    use_cases::review::{
        decide_ability_candidate, engine_adapter, facts, list_ability_candidates,
        list_match_reviews, list_reviewable_matches, package, read_match_review,
    },
    ApplicationResult,
};
use football_domain::{
    AbilityCandidateDecisionDraft, AbilityCandidateStatus, AbilityUpdateCandidateRecord,
    MatchReviewDetail, MatchReviewDraft, MatchReviewPackageCommitRequest,
    MatchReviewPackageCommitResult, MatchReviewPackageConfirmationRequest,
    MatchReviewPackageFactsCommitResult, MatchReviewPackagePreview, MatchReviewPackageReviewResult,
    MatchReviewPackageSummary, MatchReviewPackageWorkflowRecord, MatchReviewSummary,
    ReviewableMatch,
};
use uuid::Uuid;

pub(crate) struct ReviewService;

impl ReviewService {
    pub(crate) fn new() -> Self {
        Self
    }

    pub(crate) async fn generate_match_review<P: MatchReviewPort + ?Sized>(
        &self,
        port: &P,
        draft: MatchReviewDraft,
    ) -> ApplicationResult<MatchReviewDetail> {
        engine_adapter::generate_match_review(port, draft).await
    }

    pub(crate) async fn list_reviewable_matches<P: MatchReviewPort + ?Sized>(
        &self,
        port: &P,
        limit: u32,
    ) -> ApplicationResult<Vec<ReviewableMatch>> {
        list_reviewable_matches::execute(port, limit).await
    }

    pub(crate) async fn list_match_reviews<P: MatchReviewPort + ?Sized>(
        &self,
        port: &P,
        limit: u32,
    ) -> ApplicationResult<Vec<MatchReviewSummary>> {
        list_match_reviews::execute(port, limit).await
    }

    pub(crate) async fn read_match_review<P: MatchReviewPort + ?Sized>(
        &self,
        port: &P,
        review_id: Uuid,
    ) -> ApplicationResult<MatchReviewDetail> {
        read_match_review::execute(port, review_id).await
    }

    pub(crate) async fn list_ability_candidates<P: MatchReviewPort + ?Sized>(
        &self,
        port: &P,
        status: Option<AbilityCandidateStatus>,
        limit: u32,
        match_review_id: Option<Uuid>,
    ) -> ApplicationResult<Vec<AbilityUpdateCandidateRecord>> {
        list_ability_candidates::execute(port, status, limit, match_review_id).await
    }

    pub(crate) async fn decide_ability_candidate<P: MatchReviewPort + ?Sized>(
        &self,
        port: &P,
        draft: AbilityCandidateDecisionDraft,
    ) -> ApplicationResult<AbilityUpdateCandidateRecord> {
        decide_ability_candidate::execute(port, draft).await
    }

    pub(crate) async fn export_match_review_package<P>(
        &self,
        port: &P,
        output_path: String,
        match_id: Uuid,
    ) -> ApplicationResult<MatchReviewPackageSummary>
    where
        P: MatchReviewPackageSourcePort + MatchReviewPackageStatePort + ?Sized,
    {
        package::export::execute(port, output_path, match_id).await
    }

    pub(crate) async fn read_match_review_package_workflow<P>(
        &self,
        port: &P,
        match_id: Uuid,
    ) -> ApplicationResult<Option<MatchReviewPackageWorkflowRecord>>
    where
        P: MatchReviewPackageStatePort + ?Sized,
    {
        package::workflow::read_active_workflow(port, match_id).await
    }

    pub(crate) async fn preview_match_review_package<P>(
        &self,
        port: &P,
        input_path: String,
        expected_match_id: Option<Uuid>,
    ) -> ApplicationResult<MatchReviewPackagePreview>
    where
        P: MatchReviewPackageSourcePort + MatchReviewPackageStatePort + ?Sized,
    {
        package::preview::execute(port, input_path, expected_match_id).await
    }

    pub(crate) async fn confirm_match_review_package<P>(
        &self,
        port: &P,
        request: MatchReviewPackageConfirmationRequest,
    ) -> ApplicationResult<MatchReviewPackageWorkflowRecord>
    where
        P: MatchReviewPackageSourcePort + MatchReviewPackageStatePort + ?Sized,
    {
        package::workflow::confirm(port, request).await
    }

    pub(crate) async fn commit_match_review_package_facts<P>(
        &self,
        port: &P,
        package_id: Uuid,
    ) -> ApplicationResult<MatchReviewPackageFactsCommitResult>
    where
        P: MatchReviewPackageSourcePort
            + MatchReviewPackageStatePort
            + MatchReviewPackageFactsPort
            + LineupPort
            + ?Sized,
    {
        facts::commit_facts(port, package_id).await
    }

    pub(crate) async fn generate_match_review_from_package<P>(
        &self,
        port: &P,
        package_id: Uuid,
    ) -> ApplicationResult<MatchReviewPackageReviewResult>
    where
        P: MatchReviewPackageStatePort + MatchReviewPort + ?Sized,
    {
        engine_adapter::generate_review(port, package_id).await
    }

    pub(crate) async fn commit_match_review_package<P>(
        &self,
        port: &P,
        request: MatchReviewPackageCommitRequest,
    ) -> ApplicationResult<MatchReviewPackageCommitResult>
    where
        P: MatchReviewPackageSourcePort
            + MatchReviewPackageStatePort
            + MatchReviewPackageFactsPort
            + MatchReviewPort
            + LineupPort
            + ?Sized,
    {
        package::workflow::commit(port, request).await
    }
}
