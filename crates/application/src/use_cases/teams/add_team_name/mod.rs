use crate::{ports::team::TeamCatalogPort, ApplicationResult};
use football_domain::{TeamNameDraft, TeamNameRecord};
pub(crate) async fn execute<P>(port: &P, draft: TeamNameDraft) -> ApplicationResult<TeamNameRecord>
where
    P: TeamCatalogPort + ?Sized,
{
    Ok(port.add_team_name(&draft).await?)
}
