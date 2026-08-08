use crate::ports::PortResult;
use async_trait::async_trait;
use football_domain::{
    BulkDeleteResult, TeamDetail, TeamDraft, TeamForceDeletePreview, TeamForceDeleteRequest,
    TeamForceDeleteResult, TeamListPage, TeamListQuery, TeamNameDraft, TeamNameRecord, TeamOption,
    TeamProfileDraft, TeamProfileRecord, TeamRecord,
};
use uuid::Uuid;

#[async_trait]
pub trait TeamCatalogPort: Send + Sync {
    async fn create_team(&self, draft: &TeamDraft) -> PortResult<TeamRecord>;
    async fn list_team_options(
        &self,
        search: Option<&str>,
        limit: u32,
    ) -> PortResult<Vec<TeamOption>>;
    async fn list_teams(&self, query: &TeamListQuery) -> PortResult<TeamListPage>;
    async fn read_team(&self, team_id: Uuid) -> PortResult<TeamDetail>;
    async fn update_team(&self, team_id: Uuid, draft: &TeamDraft) -> PortResult<TeamRecord>;
    async fn add_team_name(&self, draft: &TeamNameDraft) -> PortResult<TeamNameRecord>;
    async fn upsert_team_profile(
        &self,
        team_id: Uuid,
        draft: &TeamProfileDraft,
    ) -> PortResult<TeamProfileRecord>;
}

#[async_trait]
pub trait TeamLifecyclePort: Send + Sync {
    async fn bulk_delete_teams(&self, team_ids: &[Uuid]) -> PortResult<BulkDeleteResult>;
    async fn preview_force_delete_team(&self, team_id: Uuid) -> PortResult<TeamForceDeletePreview>;
    async fn force_delete_team(
        &self,
        request: &TeamForceDeleteRequest,
    ) -> PortResult<TeamForceDeleteResult>;
}
