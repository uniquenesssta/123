use crate::{ports::player::EntityReferencePort, ApplicationResult};
use football_domain::{EntityMatchRequest, EntityMatchResult};
pub(crate) async fn execute<P>(
    port: &P,
    request: EntityMatchRequest,
) -> ApplicationResult<EntityMatchResult>
where
    P: EntityReferencePort + ?Sized,
{
    Ok(port.resolve_reference(&request).await?)
}
