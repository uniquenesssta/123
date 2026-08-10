use crate::{ports::postmatch::PostmatchSettlementPort, ApplicationResult};
use football_domain::PostmatchSettlementRecord;

pub(crate) async fn execute<P>(
    port: &P,
    limit: u32,
) -> ApplicationResult<Vec<PostmatchSettlementRecord>>
where
    P: PostmatchSettlementPort + ?Sized,
{
    Ok(port.list_settlements(limit).await?)
}
