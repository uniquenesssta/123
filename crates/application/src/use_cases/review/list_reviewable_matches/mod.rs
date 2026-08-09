use crate::{ports::review::MatchReviewPort, ApplicationResult};
use football_domain::ReviewableMatch;

pub(crate) async fn execute<P>(port: &P, limit: u32) -> ApplicationResult<Vec<ReviewableMatch>>
where
    P: MatchReviewPort + ?Sized,
{
    Ok(port.list_reviewable_matches(limit).await?)
}
