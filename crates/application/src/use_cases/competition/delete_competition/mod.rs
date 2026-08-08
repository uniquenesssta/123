use crate::{ports::competition::CompetitionHierarchyPort, ApplicationResult};
use uuid::Uuid;

pub(crate) async fn execute<P>(port: &P, competition_id: Uuid) -> ApplicationResult<()>
where
    P: CompetitionHierarchyPort + ?Sized,
{
    Ok(port.delete_competition(competition_id).await?)
}
