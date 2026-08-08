use crate::{ports::player::EntityReferencePort, ApplicationResult};
use football_domain::{ExternalEntityIdDraft, ExternalEntityIdRecord};
pub(crate) async fn execute<P>(
    port: &P,
    draft: ExternalEntityIdDraft,
) -> ApplicationResult<ExternalEntityIdRecord>
where
    P: EntityReferencePort + ?Sized,
{
    Ok(port.add_external_id(&draft).await?)
}
