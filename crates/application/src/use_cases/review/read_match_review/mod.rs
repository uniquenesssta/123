use crate::{ports::review::MatchReviewPort, ApplicationResult};
use football_domain::MatchReviewDetail;
use uuid::Uuid;

pub(crate) async fn execute<P>(port: &P, review_id: Uuid) -> ApplicationResult<MatchReviewDetail>
where
    P: MatchReviewPort + ?Sized,
{
    Ok(port.read_match_review(review_id).await?)
}
