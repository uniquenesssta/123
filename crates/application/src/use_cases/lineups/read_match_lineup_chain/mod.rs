use crate::{ports::lineup::LineupPort, ApplicationResult};
use football_domain::MatchLineupChain;
use uuid::Uuid;
pub(crate) async fn execute<P>(
    port: &P,
    match_id: Uuid,
    snapshot_type: String,
) -> ApplicationResult<MatchLineupChain>
where
    P: LineupPort + ?Sized,
{
    Ok(port.read_match_chain(match_id, &snapshot_type).await?)
}
