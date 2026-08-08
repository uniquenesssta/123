use crate::{ports::lineup::LineupPort, ApplicationResult};
use football_domain::{LineupPairDraft, LineupPairRecord};
pub(crate) async fn execute<P>(
    port: &P,
    draft: LineupPairDraft,
) -> ApplicationResult<LineupPairRecord>
where
    P: LineupPort + ?Sized,
{
    Ok(port.create_lineup_pair(&draft).await?)
}
