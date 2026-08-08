use crate::{ports::team::TeamLifecyclePort, ApplicationResult};
use football_domain::{TeamForceDeleteRequest, TeamForceDeleteResult};
pub(crate) async fn execute<P>(
    port: &P,
    request: TeamForceDeleteRequest,
) -> ApplicationResult<TeamForceDeleteResult>
where
    P: TeamLifecyclePort + ?Sized,
{
    Ok(port.force_delete_team(&request).await?)
}
