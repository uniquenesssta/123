use crate::{
    ports::team::{TeamCatalogPort, TeamLifecyclePort},
    use_cases::teams::{
        add_team_name, bulk_delete_teams, create_team, force_delete_team, list_team_options,
        list_teams, preview_force_delete_team, read_team, update_team, upsert_team_profile,
    },
    ApplicationResult,
};
use football_domain::{
    BulkDeleteResult, TeamDetail, TeamDraft, TeamForceDeletePreview, TeamForceDeleteRequest,
    TeamForceDeleteResult, TeamListPage, TeamListQuery, TeamNameDraft, TeamNameRecord, TeamOption,
    TeamProfileDraft, TeamProfileRecord, TeamRecord,
};
use uuid::Uuid;
pub(crate) struct TeamService;
impl TeamService {
    pub(crate) fn new() -> Self {
        Self
    }
    pub(crate) async fn create_team<P: TeamCatalogPort + ?Sized>(
        &self,
        port: &P,
        draft: TeamDraft,
    ) -> ApplicationResult<TeamRecord> {
        create_team::execute(port, draft).await
    }
    pub(crate) async fn list_team_options<P: TeamCatalogPort + ?Sized>(
        &self,
        port: &P,
        search: Option<String>,
        limit: u32,
    ) -> ApplicationResult<Vec<TeamOption>> {
        list_team_options::execute(port, search, limit).await
    }
    pub(crate) async fn list_teams<P: TeamCatalogPort + ?Sized>(
        &self,
        port: &P,
        query: TeamListQuery,
    ) -> ApplicationResult<TeamListPage> {
        list_teams::execute(port, query).await
    }
    pub(crate) async fn read_team<P: TeamCatalogPort + ?Sized>(
        &self,
        port: &P,
        team_id: Uuid,
    ) -> ApplicationResult<TeamDetail> {
        read_team::execute(port, team_id).await
    }
    pub(crate) async fn update_team<P: TeamCatalogPort + ?Sized>(
        &self,
        port: &P,
        team_id: Uuid,
        draft: TeamDraft,
    ) -> ApplicationResult<TeamRecord> {
        update_team::execute(port, team_id, draft).await
    }
    pub(crate) async fn add_team_name<P: TeamCatalogPort + ?Sized>(
        &self,
        port: &P,
        draft: TeamNameDraft,
    ) -> ApplicationResult<TeamNameRecord> {
        add_team_name::execute(port, draft).await
    }
    pub(crate) async fn upsert_team_profile<P: TeamCatalogPort + ?Sized>(
        &self,
        port: &P,
        team_id: Uuid,
        draft: TeamProfileDraft,
    ) -> ApplicationResult<TeamProfileRecord> {
        upsert_team_profile::execute(port, team_id, draft).await
    }
    pub(crate) async fn bulk_delete_teams<P: TeamLifecyclePort + ?Sized>(
        &self,
        port: &P,
        team_ids: Vec<Uuid>,
    ) -> ApplicationResult<BulkDeleteResult> {
        bulk_delete_teams::execute(port, team_ids).await
    }
    pub(crate) async fn preview_force_delete_team<P: TeamLifecyclePort + ?Sized>(
        &self,
        port: &P,
        team_id: Uuid,
    ) -> ApplicationResult<TeamForceDeletePreview> {
        preview_force_delete_team::execute(port, team_id).await
    }
    pub(crate) async fn force_delete_team<P: TeamLifecyclePort + ?Sized>(
        &self,
        port: &P,
        request: TeamForceDeleteRequest,
    ) -> ApplicationResult<TeamForceDeleteResult> {
        force_delete_team::execute(port, request).await
    }
}
