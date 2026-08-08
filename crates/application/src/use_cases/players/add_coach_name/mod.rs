use crate::{ports::player::CoachCatalogPort, ApplicationResult};
use football_domain::{CoachNameDraft, CoachNameRecord};
pub(crate) async fn execute<P>(
    port: &P,
    draft: CoachNameDraft,
) -> ApplicationResult<CoachNameRecord>
where
    P: CoachCatalogPort + ?Sized,
{
    Ok(port.add_coach_name(&draft).await?)
}
