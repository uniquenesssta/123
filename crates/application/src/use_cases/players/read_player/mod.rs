use crate::{ports::player::PlayerCatalogPort, ApplicationResult};
use football_domain::PlayerDetail;
use uuid::Uuid;
pub(crate) async fn execute<P>(port: &P, player_id: Uuid) -> ApplicationResult<PlayerDetail>
where
    P: PlayerCatalogPort + ?Sized,
{
    Ok(port.read_player(player_id).await?)
}
