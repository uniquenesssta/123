use crate::{ports::team::TeamCatalogPort, ApplicationResult};
use football_domain::{TeamDraft, TeamRecord};
use uuid::Uuid;
pub(crate) async fn execute<P>(
    port: &P,
    team_id: Uuid,
    draft: TeamDraft,
) -> ApplicationResult<TeamRecord>
where
    P: TeamCatalogPort + ?Sized,
{
    Ok(port.update_team(team_id, &draft).await?)
}
