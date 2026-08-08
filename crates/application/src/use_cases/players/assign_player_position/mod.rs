use crate::{ports::player::PlayerCatalogPort, ApplicationResult};
use football_domain::{PlayerPositionDraft, PlayerPositionRecord};
pub(crate) async fn execute<P>(
    port: &P,
    draft: PlayerPositionDraft,
) -> ApplicationResult<PlayerPositionRecord>
where
    P: PlayerCatalogPort + ?Sized,
{
    Ok(port.assign_player_position(&draft).await?)
}
