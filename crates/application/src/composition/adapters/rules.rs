use super::super::port_registry::PersistenceStore;
use super::map_persistence_error;
use crate::ports::{
    rules::{RulePackagePort, RuleRoutingPort},
    PortResult,
};
use async_trait::async_trait;
use football_domain::{
    CompetitionBindingDraft, CompetitionBindingSummary, CompetitionKind,
    ResolvedCompetitionContext, RouteDecision, RouteRequest, RulePackageDraft, RulePackageSummary,
};
use football_model_api::ModelDescriptor;
use uuid::Uuid;

#[async_trait]
impl RulePackagePort for PersistenceStore {
    async fn register_rule_package(
        &self,
        descriptor: &ModelDescriptor,
        draft: &RulePackageDraft,
    ) -> PortResult<RulePackageSummary> {
        PersistenceStore::register_rule_package(self, descriptor, draft)
            .await
            .map_err(map_persistence_error)
    }

    async fn list_rule_packages(&self) -> PortResult<Vec<RulePackageSummary>> {
        PersistenceStore::list_rule_packages(self)
            .await
            .map_err(map_persistence_error)
    }
}

#[async_trait]
impl RuleRoutingPort for PersistenceStore {
    async fn create_competition_binding(
        &self,
        draft: &CompetitionBindingDraft,
    ) -> PortResult<CompetitionBindingSummary> {
        PersistenceStore::create_competition_binding(self, draft)
            .await
            .map_err(map_persistence_error)
    }

    async fn list_competition_bindings(&self) -> PortResult<Vec<CompetitionBindingSummary>> {
        PersistenceStore::list_competition_bindings(self)
            .await
            .map_err(map_persistence_error)
    }

    async fn ensure_type_default_binding(
        &self,
        rule_package_id: Uuid,
        competition_kind: CompetitionKind,
        priority: i32,
        label: &str,
    ) -> PortResult<()> {
        PersistenceStore::ensure_type_default_binding(
            self,
            rule_package_id,
            competition_kind,
            priority,
            label,
        )
        .await
        .map(|_| ())
        .map_err(map_persistence_error)
    }

    async fn resolve_competition_context(
        &self,
        competition_id: Option<Uuid>,
        season_id: Option<Uuid>,
        stage_id: Option<Uuid>,
        competition_kind: CompetitionKind,
    ) -> PortResult<ResolvedCompetitionContext> {
        PersistenceStore::resolve_competition_context(
            self,
            competition_id,
            season_id,
            stage_id,
            competition_kind,
        )
        .await
        .map_err(map_persistence_error)
    }

    async fn resolve_route(&self, request: &RouteRequest) -> PortResult<RouteDecision> {
        PersistenceStore::resolve_route(self, request)
            .await
            .map_err(map_persistence_error)
    }
}
