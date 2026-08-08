use crate::{ports::player::CoachCatalogPort, ApplicationResult};
use football_domain::{CoachListItem, CoachListQuery};
pub(crate) async fn execute<P>(
    port: &P,
    query: CoachListQuery,
) -> ApplicationResult<Vec<CoachListItem>>
where
    P: CoachCatalogPort + ?Sized,
{
    Ok(port.list_coaches(&query).await?)
}
