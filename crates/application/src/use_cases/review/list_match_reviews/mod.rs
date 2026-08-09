use crate::{ports::review::MatchReviewPort, ApplicationResult};
use football_domain::MatchReviewSummary;

pub(crate) async fn execute<P>(port: &P, limit: u32) -> ApplicationResult<Vec<MatchReviewSummary>>
where
    P: MatchReviewPort + ?Sized,
{
    Ok(port.list_match_reviews(limit).await?)
}
