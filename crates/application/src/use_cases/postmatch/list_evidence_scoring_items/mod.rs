use crate::{ports::postmatch::PostmatchSettlementPort, ApplicationResult};
use football_domain::EvidenceScoringItemRecord;

pub(crate) async fn execute<P>(
    port: &P,
    status: Option<String>,
    limit: u32,
) -> ApplicationResult<Vec<EvidenceScoringItemRecord>>
where
    P: PostmatchSettlementPort + ?Sized,
{
    Ok(port
        .list_evidence_scoring_items(status.as_deref(), limit)
        .await?)
}
