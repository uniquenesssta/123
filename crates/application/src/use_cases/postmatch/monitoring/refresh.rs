use crate::{ports::postmatch::PostmatchMonitoringPort, ApplicationResult};
use football_domain::{PostmatchMonitoringRequest, PostmatchOverview};

pub(crate) async fn refresh_postmatch_monitoring<P>(
    port: &P,
    request: PostmatchMonitoringRequest,
) -> ApplicationResult<PostmatchOverview>
where
    P: PostmatchMonitoringPort + ?Sized,
{
    Ok(port.refresh_monitoring(&request).await?)
}
