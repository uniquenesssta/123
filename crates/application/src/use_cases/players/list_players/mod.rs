use crate::{ports::player::PlayerCatalogPort, ApplicationResult};
use football_domain::{PlayerListPage, PlayerListQuery};
pub(crate) async fn execute<P>(
    port: &P,
    query: PlayerListQuery,
) -> ApplicationResult<PlayerListPage>
where
    P: PlayerCatalogPort + ?Sized,
{
    Ok(port.list_players(&query).await?)
}
