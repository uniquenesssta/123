use crate::{ports::lineup::FormationPort, ApplicationResult};
use football_domain::{FormationUsageDistributionRecord, FormationUsageListQuery};
pub(crate) async fn execute<P>(
    port: &P,
    query: FormationUsageListQuery,
) -> ApplicationResult<Vec<FormationUsageDistributionRecord>>
where
    P: FormationPort + ?Sized,
{
    Ok(port.list_usage_distributions(&query).await?)
}
