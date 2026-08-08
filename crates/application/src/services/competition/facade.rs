use crate::{ApplicationError, ApplicationResult, ApplicationService};
use football_domain::{
    CompetitionDraft, CompetitionRecord, RoundDraft, RoundRecord, SeasonDraft, SeasonRecord,
    StageDraft, StageRecord,
};
use uuid::Uuid;

impl ApplicationService {
    pub async fn create_competition(
        &self,
        draft: CompetitionDraft,
    ) -> ApplicationResult<CompetitionRecord> {
        let session = self
            .database
            .active_session()
            .await
            .ok_or(ApplicationError::DatabaseNotConnected)?;
        self.competition.create_competition(&session, draft).await
    }

    pub async fn delete_competition(&self, competition_id: Uuid) -> ApplicationResult<()> {
        let session = self
            .database
            .active_session()
            .await
            .ok_or(ApplicationError::DatabaseNotConnected)?;
        self.competition
            .delete_competition(&session, competition_id)
            .await
    }

    pub async fn create_season(&self, draft: SeasonDraft) -> ApplicationResult<SeasonRecord> {
        let session = self
            .database
            .active_session()
            .await
            .ok_or(ApplicationError::DatabaseNotConnected)?;
        self.competition.create_season(&session, draft).await
    }

    pub async fn create_stage(&self, draft: StageDraft) -> ApplicationResult<StageRecord> {
        let session = self
            .database
            .active_session()
            .await
            .ok_or(ApplicationError::DatabaseNotConnected)?;
        self.competition.create_stage(&session, draft).await
    }

    pub async fn create_round(&self, draft: RoundDraft) -> ApplicationResult<RoundRecord> {
        let session = self
            .database
            .active_session()
            .await
            .ok_or(ApplicationError::DatabaseNotConnected)?;
        self.competition.create_round(&session, draft).await
    }
}
