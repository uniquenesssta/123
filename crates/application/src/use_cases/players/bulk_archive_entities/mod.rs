use crate::{ports::player::EntityReferencePort, ApplicationResult};
use football_domain::BulkArchiveResult;
use uuid::Uuid;
pub(crate) async fn execute<P>(
    port: &P,
    entity_type: String,
    entity_ids: Vec<Uuid>,
) -> ApplicationResult<BulkArchiveResult>
where
    P: EntityReferencePort + ?Sized,
{
    Ok(port.bulk_archive(&entity_type, &entity_ids).await?)
}
