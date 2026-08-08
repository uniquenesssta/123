use crate::{ports::team::TeamCatalogPort, ApplicationResult};
use football_domain::{TeamListPage, TeamListQuery};
pub(crate) async fn execute<P>(port: &P, query: TeamListQuery) -> ApplicationResult<TeamListPage>
where
    P: TeamCatalogPort + ?Sized,
{
    Ok(port.list_teams(&query).await?)
}
