use crate::{ApplicationError, ApplicationResult, ApplicationService};
use football_domain::{
    AbilityCandidateDecisionDraft, AbilityCandidateStatus, AbilityUpdateCandidateRecord,
    MatchReviewDetail, MatchReviewDraft, MatchReviewSummary, ReviewableMatch,
};
use uuid::Uuid;

impl ApplicationService {
    pub async fn generate_match_review(
        &self,
        mut draft: MatchReviewDraft,
    ) -> ApplicationResult<MatchReviewDetail> {
        if draft.match_id != draft.result.match_id {
            return Err(ApplicationError::Validation(
                "正式赛果与复盘比赛不一致".to_string(),
            ));
        }
        if let Some(version) = draft.review_version.as_mut() {
            *version = version.trim().to_string();
            if version.is_empty() {
                draft.review_version = None;
            }
        }
        if let Some(notes) = draft.notes.as_mut() {
            *notes = notes.trim().to_string();
        }
        let store = self.active_store().await?;
        Ok(store.generate_match_review(&draft).await?)
    }

    pub async fn list_reviewable_matches(
        &self,
        limit: u32,
    ) -> ApplicationResult<Vec<ReviewableMatch>> {
        let store = self.active_store().await?;
        Ok(store.list_reviewable_matches(limit).await?)
    }

    pub async fn list_match_reviews(
        &self,
        limit: u32,
    ) -> ApplicationResult<Vec<MatchReviewSummary>> {
        let store = self.active_store().await?;
        Ok(store.list_match_reviews(limit).await?)
    }

    pub async fn read_match_review(&self, review_id: Uuid) -> ApplicationResult<MatchReviewDetail> {
        let store = self.active_store().await?;
        Ok(store.read_match_review(review_id).await?)
    }

    pub async fn list_ability_candidates(
        &self,
        status: Option<AbilityCandidateStatus>,
        limit: u32,
        match_review_id: Option<Uuid>,
    ) -> ApplicationResult<Vec<AbilityUpdateCandidateRecord>> {
        let store = self.active_store().await?;
        Ok(store
            .list_ability_candidates(status, limit, match_review_id)
            .await?)
    }

    pub async fn decide_ability_candidate(
        &self,
        draft: AbilityCandidateDecisionDraft,
    ) -> ApplicationResult<AbilityUpdateCandidateRecord> {
        let store = self.active_store().await?;
        Ok(store.decide_ability_candidate(&draft).await?)
    }
}
