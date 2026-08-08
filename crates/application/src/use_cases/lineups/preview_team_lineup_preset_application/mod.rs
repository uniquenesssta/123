use crate::{ports::lineup::LineupPresetPort, ApplicationResult};
use football_domain::TeamLineupPresetApplicationPreview;
use uuid::Uuid;
pub(crate) async fn execute<P>(
    port: &P,
    preset_id: Uuid,
) -> ApplicationResult<TeamLineupPresetApplicationPreview>
where
    P: LineupPresetPort + ?Sized,
{
    Ok(port.preview_application(preset_id).await?)
}
