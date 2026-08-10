use crate::{ports::postmatch::PostmatchMonitoringPort, ApplicationResult};
use football_domain::{PostmatchMonitoringRequest, PostmatchOverview};

pub(crate) async fn execute<P>(
    port: &P,
    request: PostmatchMonitoringRequest,
) -> ApplicationResult<PostmatchOverview>
where
    P: PostmatchMonitoringPort + ?Sized,
{
    Ok(port.refresh_monitoring(&request).await?)
}
