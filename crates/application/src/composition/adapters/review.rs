use super::super::port_registry::{map_persistence_error, ActiveDatabase};
use crate::ports::{review::MatchReviewPort, PortResult};
use async_trait::async_trait;
use football_domain::{
    AbilityCandidateDecisionDraft, AbilityCandidateStatus, AbilityUpdateCandidateRecord,
    MatchReviewDetail, MatchReviewDraft, MatchReviewSummary, ReviewableMatch,
};
use uuid::Uuid;

#[async_trait]
impl MatchReviewPort for ActiveDatabase {
    async fn list_reviewable_matches(&self, limit: u32) -> PortResult<Vec<ReviewableMatch>> {
        self.transition_store()
            .list_reviewable_matches(limit)
            .await
            .map_err(map_persistence_error)
    }

    async fn generate_match_review(
        &self,
        draft: &MatchReviewDraft,
    ) -> PortResult<MatchReviewDetail> {
        self.transition_store()
            .generate_match_review(draft)
            .await
            .map_err(map_persistence_error)
    }

    async fn list_match_reviews(&self, limit: u32) -> PortResult<Vec<MatchReviewSummary>> {
        self.transition_store()
            .list_match_reviews(limit)
            .await
            .map_err(map_persistence_error)
    }

    async fn read_match_review(&self, review_id: Uuid) -> PortResult<MatchReviewDetail> {
        self.transition_store()
            .read_match_review(review_id)
            .await
            .map_err(map_persistence_error)
    }

    async fn list_ability_candidates(
        &self,
        status: Option<AbilityCandidateStatus>,
        limit: u32,
        match_review_id: Option<Uuid>,
    ) -> PortResult<Vec<AbilityUpdateCandidateRecord>> {
        self.transition_store()
            .list_ability_candidates(status, limit, match_review_id)
            .await
            .map_err(map_persistence_error)
    }

    async fn decide_ability_candidate(
        &self,
        draft: &AbilityCandidateDecisionDraft,
    ) -> PortResult<AbilityUpdateCandidateRecord> {
        self.transition_store()
            .decide_ability_candidate(draft)
            .await
            .map_err(map_persistence_error)
    }
}
