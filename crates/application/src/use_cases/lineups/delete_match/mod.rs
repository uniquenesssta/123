use crate::{ports::lineup::MatchCatalogPort, ApplicationResult};
use uuid::Uuid;
pub(crate) async fn execute<P>(port: &P, match_id: Uuid) -> ApplicationResult<()>
where
    P: MatchCatalogPort + ?Sized,
{
    Ok(port.delete_match(match_id).await?)
}
