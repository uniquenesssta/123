use crate::{ports::player::CoachCatalogPort, ApplicationResult};
use football_domain::{CoachDraft, CoachRecord};
pub(crate) async fn execute<P>(port: &P, draft: CoachDraft) -> ApplicationResult<CoachRecord>
where
    P: CoachCatalogPort + ?Sized,
{
    Ok(port.create_coach(&draft).await?)
}
