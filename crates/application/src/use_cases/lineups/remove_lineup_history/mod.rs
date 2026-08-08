use crate::{ports::lineup::LineupPort, ApplicationResult};
use football_domain::LineupHistoryRemovalResult;
use uuid::Uuid;
pub(crate) async fn execute<P>(
    port: &P,
    lineup_id: Uuid,
    reason: Option<String>,
) -> ApplicationResult<LineupHistoryRemovalResult>
where
    P: LineupPort + ?Sized,
{
    Ok(port.remove_history(lineup_id, reason.as_deref()).await?)
}
