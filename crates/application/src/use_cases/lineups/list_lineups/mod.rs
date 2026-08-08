use crate::{ports::lineup::LineupPort, ApplicationResult};
use football_domain::LineupRecord;
use uuid::Uuid;
pub(crate) async fn execute<P>(
    port: &P,
    match_id: Option<Uuid>,
    limit: u32,
) -> ApplicationResult<Vec<LineupRecord>>
where
    P: LineupPort + ?Sized,
{
    Ok(port.list_lineups(match_id, limit).await?)
}
