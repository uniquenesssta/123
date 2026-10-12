use super::validate_review_draft;
use crate::{
    adapters::review::engine_adapter::prepare_review_in_tx, write_audit_event, PersistenceResult,
    PostgresStore,
};
use football_domain::MatchReviewDraft;
use serde_json::json;

impl PostgresStore {
    pub async fn commit_match_review_facts(
        &self,
        draft: &MatchReviewDraft,
    ) -> PersistenceResult<()> {
        validate_review_draft(draft)?;
        let mut tx = self.pool.begin().await?;
        let preparation = prepare_review_in_tx(&mut tx, draft).await?;
        write_audit_event(
            &mut tx,
            "match_review_facts_committed",
            "match",
            Some(draft.match_id.to_string()),
            json!({
                "result": preparation.result,
                "substitution_count": draft.substitutions.len(),
                "event_count": draft.events.len(),
                "player_observation_count": draft.player_observations.len(),
            }),
        )
        .await?;
        tx.commit().await?;
        Ok(())
    }
}
