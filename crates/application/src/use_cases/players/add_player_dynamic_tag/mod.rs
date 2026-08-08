use crate::{ports::player::PlayerSignalPort, ApplicationResult};
use football_domain::{PlayerDynamicTagDraft, PlayerDynamicTagRecord};
pub(crate) async fn execute<P>(
    port: &P,
    draft: PlayerDynamicTagDraft,
) -> ApplicationResult<PlayerDynamicTagRecord>
where
    P: PlayerSignalPort + ?Sized,
{
    Ok(port.add_dynamic_tag(&draft).await?)
}
