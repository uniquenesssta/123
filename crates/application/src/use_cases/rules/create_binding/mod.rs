use crate::{ports::rules::RuleRoutingPort, ApplicationResult};
use football_domain::{CompetitionBindingDraft, CompetitionBindingSummary};

pub(crate) async fn execute<P>(
    port: &P,
    draft: CompetitionBindingDraft,
) -> ApplicationResult<CompetitionBindingSummary>
where
    P: RuleRoutingPort + ?Sized,
{
    Ok(port.create_competition_binding(&draft).await?)
}
