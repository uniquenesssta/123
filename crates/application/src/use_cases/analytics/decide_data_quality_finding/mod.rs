use crate::{ports::analytics::AnalyticsPort, ApplicationResult};
use football_domain::{DataQualityDecisionDraft, DataQualityFinding};

pub(crate) async fn execute<P>(
    port: &P,
    draft: DataQualityDecisionDraft,
) -> ApplicationResult<DataQualityFinding>
where
    P: AnalyticsPort + ?Sized,
{
    Ok(port.decide_data_quality(&draft).await?)
}
