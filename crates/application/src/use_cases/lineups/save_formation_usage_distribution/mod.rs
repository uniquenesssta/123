use crate::{ports::lineup::FormationPort, ApplicationResult};
use football_domain::{FormationUsageDistributionDraft, FormationUsageDistributionRecord};
pub(crate) async fn execute<P>(
    port: &P,
    draft: FormationUsageDistributionDraft,
) -> ApplicationResult<FormationUsageDistributionRecord>
where
    P: FormationPort + ?Sized,
{
    Ok(port.save_usage_distribution(&draft).await?)
}
