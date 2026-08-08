use crate::{
    model_registry::ModelRegistry,
    ports::rules::{RulePackagePort, RuleRoutingPort},
    use_cases::rules::{
        create_binding,
        load_catalog::{self, RulesCatalogSnapshot},
        register_built_ins, register_package,
    },
    ApplicationResult,
};
use football_domain::{
    CompetitionBindingDraft, CompetitionBindingSummary, RulePackageDraft, RulePackageSummary,
};

pub(crate) struct RulesService;

impl RulesService {
    pub(crate) fn new() -> Self {
        Self
    }

    pub(crate) async fn register_package<P>(
        &self,
        registry: &ModelRegistry,
        port: &P,
        draft: RulePackageDraft,
    ) -> ApplicationResult<RulePackageSummary>
    where
        P: RulePackagePort + RuleRoutingPort + ?Sized,
    {
        register_package::execute(registry, port, draft).await
    }

    pub(crate) async fn register_built_ins<P>(
        &self,
        registry: &ModelRegistry,
        port: &P,
    ) -> ApplicationResult<()>
    where
        P: RulePackagePort + RuleRoutingPort + ?Sized,
    {
        register_built_ins::execute(registry, port).await
    }

    pub(crate) async fn create_binding<P>(
        &self,
        port: &P,
        draft: CompetitionBindingDraft,
    ) -> ApplicationResult<CompetitionBindingSummary>
    where
        P: RuleRoutingPort + ?Sized,
    {
        create_binding::execute(port, draft).await
    }

    pub(crate) async fn load_catalog<P>(&self, port: &P) -> ApplicationResult<RulesCatalogSnapshot>
    where
        P: RulePackagePort + RuleRoutingPort + ?Sized,
    {
        load_catalog::execute(port).await
    }
}
