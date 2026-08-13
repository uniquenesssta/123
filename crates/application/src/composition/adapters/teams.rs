use super::super::port_registry::{PersistenceError, PersistenceStore};
use super::map_persistence_error;
use crate::ports::{
    team::{TeamCatalogPort, TeamLifecyclePort},
    PortError, PortErrorKind, PortResult,
};
use async_trait::async_trait;
use football_domain::{
    BulkDeleteResult, TeamDetail, TeamDraft, TeamForceDeletePreview, TeamForceDeleteRequest,
    TeamForceDeleteResult, TeamListPage, TeamListQuery, TeamNameDraft, TeamNameRecord, TeamOption,
    TeamProfileDraft, TeamProfileRecord, TeamRecord,
};
use uuid::Uuid;

#[async_trait]
impl TeamCatalogPort for PersistenceStore {
    async fn create_team(&self, draft: &TeamDraft) -> PortResult<TeamRecord> {
        self.create_team(draft).await.map_err(map_persistence_error)
    }
    async fn list_team_options(
        &self,
        search: Option<&str>,
        limit: u32,
    ) -> PortResult<Vec<TeamOption>> {
        self.list_team_options(search, limit)
            .await
            .map_err(map_persistence_error)
    }
    async fn list_teams(&self, query: &TeamListQuery) -> PortResult<TeamListPage> {
        self.list_teams(query).await.map_err(map_persistence_error)
    }
    async fn read_team(&self, team_id: Uuid) -> PortResult<TeamDetail> {
        self.read_team(team_id).await.map_err(map_persistence_error)
    }
    async fn update_team(&self, team_id: Uuid, draft: &TeamDraft) -> PortResult<TeamRecord> {
        self.update_team(team_id, draft)
            .await
            .map_err(map_persistence_error)
    }
    async fn add_team_name(&self, draft: &TeamNameDraft) -> PortResult<TeamNameRecord> {
        self.add_team_name(draft)
            .await
            .map_err(map_persistence_error)
    }
    async fn upsert_team_profile(
        &self,
        team_id: Uuid,
        draft: &TeamProfileDraft,
    ) -> PortResult<TeamProfileRecord> {
        self.upsert_team_profile(team_id, draft)
            .await
            .map_err(map_persistence_error)
    }
}

async fn run_non_send_persistence<T, F>(label: &'static str, operation: F) -> PortResult<T>
where
    T: Send + 'static,
    F: FnOnce() -> Result<T, PersistenceError> + Send + 'static,
{
    tokio::task::spawn_blocking(operation)
        .await
        .map_err(|error| {
            PortError::new(
                PortErrorKind::Infrastructure,
                format!("{label}后台任务执行失败：{error}"),
            )
        })?
        .map_err(map_persistence_error)
}

#[async_trait]
impl TeamLifecyclePort for PersistenceStore {
    async fn bulk_delete_teams(&self, team_ids: &[Uuid]) -> PortResult<BulkDeleteResult> {
        self.bulk_delete_teams(team_ids)
            .await
            .map_err(map_persistence_error)
    }

    async fn preview_force_delete_team(&self, team_id: Uuid) -> PortResult<TeamForceDeletePreview> {
        let store = PersistenceStore::clone(self);
        let runtime = tokio::runtime::Handle::current();
        run_non_send_persistence("球队强制删除预检", move || {
            runtime.block_on(store.preview_force_delete_team(team_id))
        })
        .await
    }

    async fn force_delete_team(
        &self,
        request: &TeamForceDeleteRequest,
    ) -> PortResult<TeamForceDeleteResult> {
        let store = PersistenceStore::clone(self);
        let runtime = tokio::runtime::Handle::current();
        let request = request.clone();
        run_non_send_persistence("球队强制删除", move || {
            runtime.block_on(store.force_delete_team(&request))
        })
        .await
    }
}
