use crate::{ports::analytics::AnalyticsPort, ApplicationResult};
use football_domain::{AnalyticsOverview, AnalyticsRefreshRequest};

pub(crate) async fn refresh_analytics<P>(
    port: &P,
    request: &AnalyticsRefreshRequest,
) -> ApplicationResult<AnalyticsOverview>
where
    P: AnalyticsPort + ?Sized,
{
    Ok(port.refresh(request).await?)
}
