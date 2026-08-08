use crate::{ports::lineup::LineupPresetPort, ApplicationResult};
use football_domain::TeamLineupPresetRecord;
use uuid::Uuid;
pub(crate) async fn execute<P>(
    port: &P,
    preset_id: Uuid,
) -> ApplicationResult<TeamLineupPresetRecord>
where
    P: LineupPresetPort + ?Sized,
{
    Ok(port.archive_preset(preset_id).await?)
}
