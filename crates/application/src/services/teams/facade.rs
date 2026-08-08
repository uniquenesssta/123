use crate::{ApplicationError, ApplicationResult, ApplicationService};
use football_domain::{
    BulkDeleteResult, TeamDetail, TeamDraft, TeamForceDeletePreview, TeamForceDeleteRequest,
    TeamForceDeleteResult, TeamListPage, TeamListQuery, TeamNameDraft, TeamNameRecord, TeamOption,
    TeamProfileDraft, TeamProfileRecord, TeamRecord,
};
use uuid::Uuid;
impl ApplicationService {
    async fn team_session(&self) -> ApplicationResult<crate::composition::ActiveDatabase> {
        self.database
            .active_session()
            .await
            .ok_or(ApplicationError::DatabaseNotConnected)
    }
    pub async fn create_team(&self, draft: TeamDraft) -> ApplicationResult<TeamRecord> {
        let session = self.team_session().await?;
        self.teams.create_team(&session, draft).await
    }
    pub async fn list_team_options(
        &self,
        search: Option<String>,
        limit: u32,
    ) -> ApplicationResult<Vec<TeamOption>> {
        let session = self.team_session().await?;
        self.teams.list_team_options(&session, search, limit).await
    }
    pub async fn list_teams(&self, query: TeamListQuery) -> ApplicationResult<TeamListPage> {
        let session = self.team_session().await?;
        self.teams.list_teams(&session, query).await
    }
    pub async fn read_team(&self, team_id: Uuid) -> ApplicationResult<TeamDetail> {
        let session = self.team_session().await?;
        self.teams.read_team(&session, team_id).await
    }
    pub async fn update_team(
        &self,
        team_id: Uuid,
        draft: TeamDraft,
    ) -> ApplicationResult<TeamRecord> {
        let session = self.team_session().await?;
        self.teams.update_team(&session, team_id, draft).await
    }
    pub async fn add_team_name(&self, draft: TeamNameDraft) -> ApplicationResult<TeamNameRecord> {
        let session = self.team_session().await?;
        self.teams.add_team_name(&session, draft).await
    }
    pub async fn upsert_team_profile(
        &self,
        team_id: Uuid,
        draft: TeamProfileDraft,
    ) -> ApplicationResult<TeamProfileRecord> {
        let session = self.team_session().await?;
        self.teams
            .upsert_team_profile(&session, team_id, draft)
            .await
    }
    pub async fn bulk_delete_teams(
        &self,
        team_ids: Vec<Uuid>,
    ) -> ApplicationResult<BulkDeleteResult> {
        let session = self.team_session().await?;
        self.teams.bulk_delete_teams(&session, team_ids).await
    }
    pub async fn preview_force_delete_team(
        &self,
        team_id: Uuid,
    ) -> ApplicationResult<TeamForceDeletePreview> {
        let session = self.team_session().await?;
        self.teams
            .preview_force_delete_team(&session, team_id)
            .await
    }
    pub async fn force_delete_team(
        &self,
        request: TeamForceDeleteRequest,
    ) -> ApplicationResult<TeamForceDeleteResult> {
        let session = self.team_session().await?;
        self.teams.force_delete_team(&session, request).await
    }
}
