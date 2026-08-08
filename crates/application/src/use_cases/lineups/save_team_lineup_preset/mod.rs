use crate::{ports::lineup::LineupPresetPort, ApplicationResult};
use football_domain::{TeamLineupPresetDraft, TeamLineupPresetRecord};
pub(crate) async fn execute<P>(
    port: &P,
    draft: TeamLineupPresetDraft,
) -> ApplicationResult<TeamLineupPresetRecord>
where
    P: LineupPresetPort + ?Sized,
{
    Ok(port.save_preset(&draft).await?)
}
