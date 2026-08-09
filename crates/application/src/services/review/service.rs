use crate::{
    ports::review::MatchReviewPort,
    use_cases::review::{
        decide_ability_candidate, generate_match_review, list_ability_candidates,
        list_match_reviews, list_reviewable_matches, read_match_review,
    },
    ApplicationResult,
};
use football_domain::{
    AbilityCandidateDecisionDraft, AbilityCandidateStatus, AbilityUpdateCandidateRecord,
    MatchReviewDetail, MatchReviewDraft, MatchReviewSummary, ReviewableMatch,
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
        generate_match_review::execute(port, draft).await
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
}
