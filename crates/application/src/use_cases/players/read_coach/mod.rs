use crate::{ports::player::CoachCatalogPort, ApplicationResult};
use football_domain::CoachDetail;
use uuid::Uuid;
pub(crate) async fn execute<P>(port: &P, coach_id: Uuid) -> ApplicationResult<CoachDetail>
where
    P: CoachCatalogPort + ?Sized,
{
    Ok(port.read_coach(coach_id).await?)
}
