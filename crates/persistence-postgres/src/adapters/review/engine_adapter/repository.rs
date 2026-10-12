use super::{preparation::prepare_review_in_tx, report::persist_calculation_in_tx};
use crate::{
    adapters::review::facts::validate_review_draft, write_audit_event, PersistenceResult,
    PostgresStore,
};
use football_domain::{MatchReviewDetail, MatchReviewDraft};
use football_review_engine::calculate_review;
use serde_json::json;

impl PostgresStore {
    pub async fn generate_match_review(
        &self,
        draft: &MatchReviewDraft,
    ) -> PersistenceResult<MatchReviewDetail> {
        validate_review_draft(draft)?;
        let mut tx = self.pool.begin().await?;
        let preparation = prepare_review_in_tx(&mut tx, draft).await?;
        let calculation = calculate_review(&preparation);
        let review_id =
            persist_calculation_in_tx(&mut tx, draft, &preparation, &calculation).await?;
        write_audit_event(
            &mut tx,
            "match_review_generated",
            "match_review",
            Some(review_id.to_string()),
            json!({
                "match_id": draft.match_id,
                "review_version": draft.review_version,
                "calculation_version": calculation.calculation_version,
                "player_count": calculation.player_reviews.len(),
                "team_count": calculation.team_reviews.len(),
                "ability_candidate_count": calculation.player_reviews.iter().map(|item| item.ability_candidates.len()).sum::<usize>(),
            }),
        )
        .await?;
        tx.commit().await?;
        self.read_match_review(review_id).await
    }
}
