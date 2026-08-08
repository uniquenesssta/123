use crate::ports::PortResult;
use async_trait::async_trait;
use football_domain::{
    CompetitionBindingDraft, CompetitionBindingSummary, CompetitionKind, RouteDecision,
    RouteRequest, RulePackageDraft, RulePackageSummary,
};
use football_model_api::ModelDescriptor;
use uuid::Uuid;

#[async_trait]
pub trait RulePackagePort: Send + Sync {
    async fn register_rule_package(
        &self,
        descriptor: &ModelDescriptor,
        draft: &RulePackageDraft,
    ) -> PortResult<RulePackageSummary>;
    async fn list_rule_packages(&self) -> PortResult<Vec<RulePackageSummary>>;
}

#[async_trait]
pub trait RuleRoutingPort: Send + Sync {
    async fn create_competition_binding(
        &self,
        draft: &CompetitionBindingDraft,
    ) -> PortResult<CompetitionBindingSummary>;
    async fn list_competition_bindings(&self) -> PortResult<Vec<CompetitionBindingSummary>>;
    async fn ensure_type_default_binding(
        &self,
        rule_package_id: Uuid,
        competition_kind: CompetitionKind,
        priority: i32,
        label: &str,
    ) -> PortResult<()>;
    async fn resolve_route(&self, request: &RouteRequest) -> PortResult<RouteDecision>;
}
