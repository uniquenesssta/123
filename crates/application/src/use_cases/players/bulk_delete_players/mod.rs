use crate::{ports::player::PlayerCatalogPort, ApplicationResult};
use football_domain::BulkDeleteResult;
use uuid::Uuid;
pub(crate) async fn execute<P>(
    port: &P,
    player_ids: Vec<Uuid>,
) -> ApplicationResult<BulkDeleteResult>
where
    P: PlayerCatalogPort + ?Sized,
{
    Ok(port.bulk_delete_players(&player_ids).await?)
}
