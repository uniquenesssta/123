use crate::{ports::analytics::AnalyticsPort, ApplicationResult};
use football_domain::AnalyticsOverview;

pub(crate) async fn analytics_overview<P>(port: &P) -> ApplicationResult<AnalyticsOverview>
where
    P: AnalyticsPort + ?Sized,
{
    Ok(port.overview().await?)
}
