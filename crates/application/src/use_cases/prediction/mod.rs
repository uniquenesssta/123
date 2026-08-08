use crate::ports::{
    lineup::{LineupPort, MatchCatalogPort},
    prediction::{ModelRunPort, PredictionInputPort},
    rules::RuleRoutingPort,
};

pub(crate) mod dry_run_default_fixture;
pub(crate) mod execute_prediction;
pub(crate) mod execute_prediction_from_match;
pub(crate) mod hide_run_from_history;
pub(crate) mod inspect_match_prediction_readiness;
pub(crate) mod list_p4_freeze_task_events;
pub(crate) mod list_p4_freeze_tasks;
pub(crate) mod list_recent_runs;
pub(crate) mod p4_freeze_readiness;
pub(crate) mod plan_p4_horizons;
pub(crate) mod preview_route;
pub(crate) mod read_p4_freeze_task;
pub(crate) mod read_p4_match_workspace;
pub(crate) mod read_p4_task_workspace;
pub(crate) mod read_run;
pub(crate) mod shared;

pub(crate) trait PredictionAccess:
    PredictionInputPort + ModelRunPort + RuleRoutingPort + MatchCatalogPort + LineupPort
{
}

impl<T> PredictionAccess for T where
    T: PredictionInputPort + ModelRunPort + RuleRoutingPort + MatchCatalogPort + LineupPort
{
}

#[cfg(test)]
mod tests;

pub(crate) trait P4PlanningAccess:
    crate::ports::prediction::PredictionWorkflowPort
    + crate::ports::rules::RuleRoutingPort
    + crate::ports::research::ResearchArtifactPort
    + crate::ports::analytics::JobQueuePort
{
}

impl<T> P4PlanningAccess for T where
    T: crate::ports::prediction::PredictionWorkflowPort
        + crate::ports::rules::RuleRoutingPort
        + crate::ports::research::ResearchArtifactPort
        + crate::ports::analytics::JobQueuePort
{
}
