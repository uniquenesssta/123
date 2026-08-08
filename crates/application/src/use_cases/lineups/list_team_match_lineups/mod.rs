use crate::{ports::lineup::LineupPort, ApplicationResult};
use football_domain::TeamMatchLineupHistoryItem;
use uuid::Uuid;
pub(crate) async fn execute<P>(
    port: &P,
    team_id: Uuid,
    limit: u32,
) -> ApplicationResult<Vec<TeamMatchLineupHistoryItem>>
where
    P: LineupPort + ?Sized,
{
    Ok(port.list_team_match_lineups(team_id, limit).await?)
}
