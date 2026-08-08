use crate::{ports::lineup::LineupPresetPort, ApplicationResult};
use football_domain::TeamLineupPresetRecord;
use uuid::Uuid;
pub(crate) async fn execute<P>(
    port: &P,
    team_id: Uuid,
    include_archived: bool,
) -> ApplicationResult<Vec<TeamLineupPresetRecord>>
where
    P: LineupPresetPort + ?Sized,
{
    Ok(port.list_presets(team_id, include_archived).await?)
}
