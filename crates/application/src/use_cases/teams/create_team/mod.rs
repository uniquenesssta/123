use crate::{ports::team::TeamCatalogPort, ApplicationResult};
use football_domain::{TeamDraft, TeamRecord};
pub(crate) async fn execute<P>(port: &P, draft: TeamDraft) -> ApplicationResult<TeamRecord>
where
    P: TeamCatalogPort + ?Sized,
{
    Ok(port.create_team(&draft).await?)
}
