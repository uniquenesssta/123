use crate::{ports::player::CoachCatalogPort, ApplicationResult};
use football_domain::{TeamCoachPeriodDraft, TeamCoachPeriodRecord};
pub(crate) async fn execute<P>(
    port: &P,
    draft: TeamCoachPeriodDraft,
) -> ApplicationResult<TeamCoachPeriodRecord>
where
    P: CoachCatalogPort + ?Sized,
{
    Ok(port.add_team_coach_period(&draft).await?)
}
