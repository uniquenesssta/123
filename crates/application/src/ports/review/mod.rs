use crate::ports::PortResult;
use async_trait::async_trait;
use football_domain::{
    AbilityCandidateDecisionDraft, AbilityCandidateStatus, AbilityUpdateCandidateRecord,
    MatchReviewDetail, MatchReviewDraft, MatchReviewPackageCommitRequest,
    MatchReviewPackageCommitResult, MatchReviewPackageConfirmationRequest,
    MatchReviewPackagePreview, MatchReviewPackageWorkflowRecord, MatchReviewSummary,
    ReviewableMatch,
};
use uuid::Uuid;

#[async_trait]
pub trait MatchReviewPort: Send + Sync {
    async fn list_reviewable_matches(&self, limit: u32) -> PortResult<Vec<ReviewableMatch>>;
    async fn generate_match_review(
        &self,
        draft: &MatchReviewDraft,
    ) -> PortResult<MatchReviewDetail>;
    async fn list_match_reviews(&self, limit: u32) -> PortResult<Vec<MatchReviewSummary>>;
    async fn read_match_review(&self, review_id: Uuid) -> PortResult<MatchReviewDetail>;
    async fn list_ability_candidates(
        &self,
        status: Option<AbilityCandidateStatus>,
        limit: u32,
        match_review_id: Option<Uuid>,
    ) -> PortResult<Vec<AbilityUpdateCandidateRecord>>;
    async fn decide_ability_candidate(
        &self,
        draft: &AbilityCandidateDecisionDraft,
    ) -> PortResult<AbilityUpdateCandidateRecord>;
}

#[async_trait]
pub trait MatchReviewWorkflowPort: Send + Sync {
    async fn read_active_workflow(
        &self,
        match_id: Uuid,
    ) -> PortResult<Option<MatchReviewPackageWorkflowRecord>>;
    async fn read_package_preview(
        &self,
        workflow_id: Uuid,
    ) -> PortResult<MatchReviewPackagePreview>;
    async fn confirm_workflow(
        &self,
        request: &MatchReviewPackageConfirmationRequest,
    ) -> PortResult<MatchReviewPackageWorkflowRecord>;
    async fn commit_package_facts(
        &self,
        request: &MatchReviewPackageCommitRequest,
    ) -> PortResult<MatchReviewPackageCommitResult>;
}
