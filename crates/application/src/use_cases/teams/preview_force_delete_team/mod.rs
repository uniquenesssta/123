use crate::{ports::team::TeamLifecyclePort, ApplicationResult};
use football_domain::TeamForceDeletePreview;
use uuid::Uuid;
pub(crate) async fn execute<P>(port: &P, team_id: Uuid) -> ApplicationResult<TeamForceDeletePreview>
where
    P: TeamLifecyclePort + ?Sized,
{
    Ok(port.preview_force_delete_team(team_id).await?)
}
