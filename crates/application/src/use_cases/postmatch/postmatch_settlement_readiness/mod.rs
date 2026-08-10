use crate::{ports::postmatch::PostmatchSettlementPort, ApplicationResult};
use football_domain::PostmatchSettlementReadiness;
use uuid::Uuid;

pub(crate) async fn execute<P>(
    port: &P,
    match_review_id: Uuid,
) -> ApplicationResult<PostmatchSettlementReadiness>
where
    P: PostmatchSettlementPort + ?Sized,
{
    Ok(port.readiness(match_review_id).await?)
}
