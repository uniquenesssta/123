use crate::{ApplicationError, ApplicationResult, ApplicationService};
use football_domain::{
    AbilityCandidateDecisionDraft, AbilityCandidateStatus, AbilityUpdateCandidateRecord,
    MatchReviewDetail, MatchReviewDraft, MatchReviewSummary, ReviewableMatch,
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
}
