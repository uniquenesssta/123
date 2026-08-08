use crate::{ports::player::EntityReferencePort, ApplicationResult};
use football_domain::EntityDeletionCheck;
use uuid::Uuid;
pub(crate) async fn execute<P>(
    port: &P,
    entity_type: String,
    entity_id: Uuid,
) -> ApplicationResult<EntityDeletionCheck>
where
    P: EntityReferencePort + ?Sized,
{
    Ok(port.check_deletion(&entity_type, entity_id).await?)
}
