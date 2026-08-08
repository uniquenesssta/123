use crate::{ports::player::PlayerCatalogPort, ApplicationResult};
use football_domain::{PlayerNameDraft, PlayerNameRecord};
pub(crate) async fn execute<P>(
    port: &P,
    draft: PlayerNameDraft,
) -> ApplicationResult<PlayerNameRecord>
where
    P: PlayerCatalogPort + ?Sized,
{
    Ok(port.add_player_name(&draft).await?)
}
