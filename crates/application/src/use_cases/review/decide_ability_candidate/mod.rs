use crate::{ports::review::MatchReviewPort, ApplicationResult};
use football_domain::{AbilityCandidateDecisionDraft, AbilityUpdateCandidateRecord};

pub(crate) async fn execute<P>(
    port: &P,
    draft: AbilityCandidateDecisionDraft,
) -> ApplicationResult<AbilityUpdateCandidateRecord>
where
    P: MatchReviewPort + ?Sized,
{
    Ok(port.decide_ability_candidate(&draft).await?)
}
