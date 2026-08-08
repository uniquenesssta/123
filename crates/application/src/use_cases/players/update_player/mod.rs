use crate::{ports::player::PlayerCatalogPort, ApplicationResult};
use football_domain::{PlayerDraft, PlayerRecord};
use uuid::Uuid;
pub(crate) async fn execute<P>(
    port: &P,
    player_id: Uuid,
    draft: PlayerDraft,
) -> ApplicationResult<PlayerRecord>
where
    P: PlayerCatalogPort + ?Sized,
{
    Ok(port.update_player(player_id, &draft).await?)
}
