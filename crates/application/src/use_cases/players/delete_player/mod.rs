use crate::{ports::player::PlayerCatalogPort, ApplicationResult};
use uuid::Uuid;
pub(crate) async fn execute<P>(port: &P, player_id: Uuid) -> ApplicationResult<()>
where
    P: PlayerCatalogPort + ?Sized,
{
    Ok(port.delete_player(player_id).await?)
}
