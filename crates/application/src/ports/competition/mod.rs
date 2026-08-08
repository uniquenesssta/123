use crate::ports::PortResult;
use async_trait::async_trait;
use football_domain::{
    CompetitionDraft, CompetitionRecord, RoundDraft, RoundRecord, SeasonDraft, SeasonRecord,
    StageDraft, StageRecord,
};
use uuid::Uuid;

#[async_trait]
pub trait CompetitionHierarchyPort: Send + Sync {
    async fn create_competition(&self, draft: &CompetitionDraft) -> PortResult<CompetitionRecord>;
    async fn delete_competition(&self, competition_id: Uuid) -> PortResult<()>;
    async fn list_competitions(&self) -> PortResult<Vec<CompetitionRecord>>;
    async fn create_season(&self, draft: &SeasonDraft) -> PortResult<SeasonRecord>;
    async fn list_seasons(&self) -> PortResult<Vec<SeasonRecord>>;
    async fn create_stage(&self, draft: &StageDraft) -> PortResult<StageRecord>;
    async fn list_stages(&self) -> PortResult<Vec<StageRecord>>;
    async fn create_round(&self, draft: &RoundDraft) -> PortResult<RoundRecord>;
    async fn list_rounds(&self) -> PortResult<Vec<RoundRecord>>;
}
