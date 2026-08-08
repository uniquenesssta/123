use crate::{ports::lineup::MatchCatalogPort, ApplicationResult};
use football_domain::{MatchDraft, MatchRecord};
pub(crate) async fn execute<P>(port: &P, draft: MatchDraft) -> ApplicationResult<MatchRecord>
where
    P: MatchCatalogPort + ?Sized,
{
    Ok(port.create_match(&draft).await?)
}
