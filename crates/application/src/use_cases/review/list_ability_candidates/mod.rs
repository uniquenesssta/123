use crate::{ports::review::MatchReviewPort, ApplicationResult};
use football_domain::{AbilityCandidateStatus, AbilityUpdateCandidateRecord};
use uuid::Uuid;

pub(crate) async fn execute<P>(
    port: &P,
    status: Option<AbilityCandidateStatus>,
    limit: u32,
    match_review_id: Option<Uuid>,
) -> ApplicationResult<Vec<AbilityUpdateCandidateRecord>>
where
    P: MatchReviewPort + ?Sized,
{
    Ok(port
        .list_ability_candidates(status, limit, match_review_id)
        .await?)
}
