use crate::{ports::lineup::FormationPort, ApplicationResult};
use football_domain::{FormationDistributionQuery, ResolvedFormationDistribution};
pub(crate) async fn execute<P>(
    port: &P,
    query: FormationDistributionQuery,
) -> ApplicationResult<ResolvedFormationDistribution>
where
    P: FormationPort + ?Sized,
{
    Ok(port.resolve_distribution(&query).await?)
}
