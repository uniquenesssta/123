use crate::{
    ports::rules::{RulePackagePort, RuleRoutingPort},
    ApplicationResult,
};
use football_domain::{CompetitionBindingSummary, RulePackageSummary};

pub(crate) struct RulesCatalogSnapshot {
    pub(crate) rule_packages: Vec<RulePackageSummary>,
    pub(crate) competition_bindings: Vec<CompetitionBindingSummary>,
}

pub(crate) async fn execute<P>(port: &P) -> ApplicationResult<RulesCatalogSnapshot>
where
    P: RulePackagePort + RuleRoutingPort + ?Sized,
{
    Ok(RulesCatalogSnapshot {
        rule_packages: port.list_rule_packages().await?,
        competition_bindings: port.list_competition_bindings().await?,
    })
}
