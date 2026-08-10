use crate::{ports::postmatch::PostmatchSettlementPort, ApplicationResult};
use football_domain::{EvidenceScoringDecisionDraft, EvidenceScoringItemRecord};

pub(crate) async fn execute<P>(
    port: &P,
    draft: EvidenceScoringDecisionDraft,
) -> ApplicationResult<EvidenceScoringItemRecord>
where
    P: PostmatchSettlementPort + ?Sized,
{
    Ok(port.decide_evidence_scoring_item(&draft).await?)
}
