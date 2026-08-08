use crate::{ports::lineup::LineupPort, ApplicationResult};
use football_domain::LineupRecord;
use uuid::Uuid;
pub(crate) async fn execute<P>(port: &P, lineup_id: Uuid) -> ApplicationResult<LineupRecord>
where
    P: LineupPort + ?Sized,
{
    Ok(port.read_lineup(lineup_id).await?)
}
