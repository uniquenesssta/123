use crate::{ports::lineup::FormationPort, ApplicationResult};
use football_domain::FormationRecord;
pub(crate) async fn execute<P>(
    port: &P,
    active_only: bool,
) -> ApplicationResult<Vec<FormationRecord>>
where
    P: FormationPort + ?Sized,
{
    Ok(port.list_formations(active_only).await?)
}
