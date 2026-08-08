use crate::{ports::lineup::LineupPresetPort, ApplicationResult};
use football_domain::TeamLineupPresetRecord;
use uuid::Uuid;
pub(crate) async fn execute<P>(
    port: &P,
    preset_id: Uuid,
    name: String,
) -> ApplicationResult<TeamLineupPresetRecord>
where
    P: LineupPresetPort + ?Sized,
{
    Ok(port.duplicate_preset(preset_id, &name).await?)
}
