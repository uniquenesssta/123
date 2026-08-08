use crate::{ports::team::TeamCatalogPort, ApplicationResult};
use football_domain::{TeamProfileDraft, TeamProfileRecord};
use uuid::Uuid;
pub(crate) async fn execute<P>(
    port: &P,
    team_id: Uuid,
    draft: TeamProfileDraft,
) -> ApplicationResult<TeamProfileRecord>
where
    P: TeamCatalogPort + ?Sized,
{
    Ok(port.upsert_team_profile(team_id, &draft).await?)
}
