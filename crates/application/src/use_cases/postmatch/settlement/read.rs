use crate::{ports::postmatch::PostmatchSettlementPort, ApplicationResult};
use football_domain::{PostmatchSettlementReadiness, PostmatchSettlementRecord};
use uuid::Uuid;

pub(crate) async fn postmatch_settlement_readiness<P>(
    port: &P,
    match_review_id: Uuid,
) -> ApplicationResult<PostmatchSettlementReadiness>
where
    P: PostmatchSettlementPort + ?Sized,
{
    Ok(port.readiness(match_review_id).await?)
}

pub(crate) async fn list_postmatch_settlements<P>(
    port: &P,
    limit: u32,
) -> ApplicationResult<Vec<PostmatchSettlementRecord>>
where
    P: PostmatchSettlementPort + ?Sized,
{
    Ok(port.list_settlements(limit).await?)
}
