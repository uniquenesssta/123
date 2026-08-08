use crate::{
    ports::competition::CompetitionHierarchyPort,
    use_cases::competition::{
        create_competition, create_round, create_season, create_stage, delete_competition,
        load_hierarchy::{self, CompetitionHierarchySnapshot},
    },
    ApplicationResult,
};
use football_domain::{
    CompetitionDraft, CompetitionRecord, RoundDraft, RoundRecord, SeasonDraft, SeasonRecord,
    StageDraft, StageRecord,
};
use uuid::Uuid;

pub(crate) struct CompetitionService;

impl CompetitionService {
    pub(crate) fn new() -> Self {
        Self
    }

    pub(crate) async fn create_competition<P>(
        &self,
        port: &P,
        draft: CompetitionDraft,
    ) -> ApplicationResult<CompetitionRecord>
    where
        P: CompetitionHierarchyPort + ?Sized,
    {
        create_competition::execute(port, draft).await
    }

    pub(crate) async fn delete_competition<P>(
        &self,
        port: &P,
        competition_id: Uuid,
    ) -> ApplicationResult<()>
    where
        P: CompetitionHierarchyPort + ?Sized,
    {
        delete_competition::execute(port, competition_id).await
    }

    pub(crate) async fn create_season<P>(
        &self,
        port: &P,
        draft: SeasonDraft,
    ) -> ApplicationResult<SeasonRecord>
    where
        P: CompetitionHierarchyPort + ?Sized,
    {
        create_season::execute(port, draft).await
    }

    pub(crate) async fn create_stage<P>(
        &self,
        port: &P,
        draft: StageDraft,
    ) -> ApplicationResult<StageRecord>
    where
        P: CompetitionHierarchyPort + ?Sized,
    {
        create_stage::execute(port, draft).await
    }

    pub(crate) async fn create_round<P>(
        &self,
        port: &P,
        draft: RoundDraft,
    ) -> ApplicationResult<RoundRecord>
    where
        P: CompetitionHierarchyPort + ?Sized,
    {
        create_round::execute(port, draft).await
    }

    pub(crate) async fn load_hierarchy<P>(
        &self,
        port: &P,
    ) -> ApplicationResult<CompetitionHierarchySnapshot>
    where
        P: CompetitionHierarchyPort + ?Sized,
    {
        load_hierarchy::execute(port).await
    }
}
