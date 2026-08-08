use crate::{ports::team::TeamCatalogPort, ApplicationResult};
use football_domain::TeamDetail;
use uuid::Uuid;
pub(crate) async fn execute<P>(port: &P, team_id: Uuid) -> ApplicationResult<TeamDetail>
where
    P: TeamCatalogPort + ?Sized,
{
    Ok(port.read_team(team_id).await?)
}
