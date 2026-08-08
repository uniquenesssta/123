use crate::{ports::competition::CompetitionHierarchyPort, ApplicationResult};
use football_domain::{CompetitionRecord, RoundRecord, SeasonRecord, StageRecord};

pub(crate) struct CompetitionHierarchySnapshot {
    pub(crate) competitions: Vec<CompetitionRecord>,
    pub(crate) seasons: Vec<SeasonRecord>,
    pub(crate) stages: Vec<StageRecord>,
    pub(crate) rounds: Vec<RoundRecord>,
}

pub(crate) async fn execute<P>(port: &P) -> ApplicationResult<CompetitionHierarchySnapshot>
where
    P: CompetitionHierarchyPort + ?Sized,
{
    Ok(CompetitionHierarchySnapshot {
        competitions: port.list_competitions().await?,
        seasons: port.list_seasons().await?,
        stages: port.list_stages().await?,
        rounds: port.list_rounds().await?,
    })
}
