use crate::{ports::player::PlayerCatalogPort, ApplicationResult};
use football_domain::{PlayerDraft, PlayerRecord};
pub(crate) async fn execute<P>(port: &P, draft: PlayerDraft) -> ApplicationResult<PlayerRecord>
where
    P: PlayerCatalogPort + ?Sized,
{
    Ok(port.create_player(&draft).await?)
}
