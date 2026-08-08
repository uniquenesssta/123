use crate::{ports::player::EntityReferencePort, ApplicationResult};
use football_domain::{EntityReferenceQuery, EntityReferenceRecord};
pub(crate) async fn execute<P>(
    port: &P,
    query: EntityReferenceQuery,
) -> ApplicationResult<Vec<EntityReferenceRecord>>
where
    P: EntityReferencePort + ?Sized,
{
    Ok(port.list_references(&query).await?)
}
