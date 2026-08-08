use crate::{ports::player::PlayerSignalPort, ApplicationResult};
use football_domain::{PlayerAvailabilityDraft, PlayerAvailabilityRecord};
pub(crate) async fn execute<P>(
    port: &P,
    draft: PlayerAvailabilityDraft,
) -> ApplicationResult<PlayerAvailabilityRecord>
where
    P: PlayerSignalPort + ?Sized,
{
    Ok(port.add_availability(&draft).await?)
}
