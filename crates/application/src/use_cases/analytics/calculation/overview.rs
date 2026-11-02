use crate::{ports::analytics::AnalyticsPort, ApplicationResult};
use football_domain::AnalyticsOverview;

pub(crate) async fn analytics_overview<P>(
    port: &P,
    snapshot_id: Option<uuid::Uuid>,
) -> ApplicationResult<AnalyticsOverview>
where
    P: AnalyticsPort + ?Sized,
{
    Ok(port.overview(snapshot_id).await?)
}
