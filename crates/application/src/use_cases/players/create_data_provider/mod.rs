use crate::{ports::player::EntityReferencePort, ApplicationResult};
use football_domain::{DataProviderDraft, DataProviderRecord};
pub(crate) async fn execute<P>(
    port: &P,
    draft: DataProviderDraft,
) -> ApplicationResult<DataProviderRecord>
where
    P: EntityReferencePort + ?Sized,
{
    Ok(port.create_data_provider(&draft).await?)
}
