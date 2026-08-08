use crate::{ports::team::TeamLifecyclePort, ApplicationResult};
use football_domain::BulkDeleteResult;
use uuid::Uuid;
pub(crate) async fn execute<P>(port: &P, team_ids: Vec<Uuid>) -> ApplicationResult<BulkDeleteResult>
where
    P: TeamLifecyclePort + ?Sized,
{
    Ok(port.bulk_delete_teams(&team_ids).await?)
}
