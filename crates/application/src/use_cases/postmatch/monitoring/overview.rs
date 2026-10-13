use crate::{ports::postmatch::PostmatchMonitoringPort, ApplicationResult};
use football_domain::PostmatchOverview;

pub(crate) async fn postmatch_overview<P>(
    port: &P,
    limit: u32,
) -> ApplicationResult<PostmatchOverview>
where
    P: PostmatchMonitoringPort + ?Sized,
{
    Ok(port.overview(limit).await?)
}
