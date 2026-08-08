use crate::{ports::lineup::LineupPresetPort, ApplicationResult};
use uuid::Uuid;
pub(crate) async fn execute<P>(port: &P, preset_id: Uuid) -> ApplicationResult<()>
where
    P: LineupPresetPort + ?Sized,
{
    Ok(port.delete_preset(preset_id).await?)
}
