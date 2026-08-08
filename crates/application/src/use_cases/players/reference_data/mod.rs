use crate::{ports::player::PlayerCatalogPort, ApplicationResult};
use football_domain::PlayerCatalogReferenceData;
pub(crate) async fn execute<P>(port: &P) -> ApplicationResult<PlayerCatalogReferenceData>
where
    P: PlayerCatalogPort + ?Sized,
{
    Ok(port.reference_data().await?)
}
