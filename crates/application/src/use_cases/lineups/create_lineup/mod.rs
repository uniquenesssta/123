use crate::{ports::lineup::LineupPort, ApplicationResult};
use football_domain::{LineupDraft, LineupRecord};
pub(crate) async fn execute<P>(port: &P, draft: LineupDraft) -> ApplicationResult<LineupRecord>
where
    P: LineupPort + ?Sized,
{
    Ok(port.create_lineup(&draft).await?)
}
