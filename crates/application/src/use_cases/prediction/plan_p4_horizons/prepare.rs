use crate::built_in_artifacts::{
    P4_RESEARCH_SCHEMA_ARTIFACT_VERSION as RESEARCH_SCHEMA_VERSION,
    P4_RESEARCH_SCHEMA_KEY as RESEARCH_SCHEMA_KEY,
    P4_SNAPSHOT_SCHEMA_ARTIFACT_VERSION as SNAPSHOT_SCHEMA_VERSION,
    P4_SNAPSHOT_SCHEMA_KEY as SNAPSHOT_SCHEMA_KEY,
};
use crate::use_cases::prediction::shared::p4_planning::{
    is_p4_model, validate_requested_fact_keys,
};
use crate::use_cases::prediction::P4PlanningAccess;
use crate::{ApplicationError, ApplicationResult};
use football_domain::{
    P4Horizon, P4PlanningMatchContext, PlanP4HorizonsCommand, RouteDecision, SchemaVersionRecord,
};

pub(super) struct PreparedPlan {
    pub(super) context: P4PlanningMatchContext,
    pub(super) decision: RouteDecision,
    pub(super) requested_fact_keys: Vec<String>,
    pub(super) research_schema: SchemaVersionRecord,
    pub(super) snapshot_schema: SchemaVersionRecord,
}

pub(super) async fn prepare<P: P4PlanningAccess + ?Sized>(
    port: &P,
    command: PlanP4HorizonsCommand,
) -> ApplicationResult<PreparedPlan> {
    let store = port;
    let context = store.planning_match_context(command.match_id).await?;
    let scope = store
        .resolve_competition_context(
            context.competition_id,
            context.season_id,
            context.stage_id,
            context.competition_kind,
        )
        .await?;
    let decision = store
        .resolve_route(&football_domain::RouteRequest {
            competition_id: scope.competition_id,
            season_id: scope.season_id,
            stage_id: scope.stage_id,
            competition_kind: scope.competition_kind,
            kickoff_time: context.kickoff_at,
            preferred_model_family: Some("p4".to_string()),
            preferred_model_id: None,
            explicit_rule_package_id: Some(command.explicit_rule_package_id),
        })
        .await?;
    if !is_p4_model(&decision.model_id) {
        return Err(ApplicationError::Validation(format!(
            "接入点F只允许显式选择P4规则包，当前模型为 {}",
            decision.model_id
        )));
    }
    for horizon in P4Horizon::CANONICAL {
        if !decision
            .routing
            .supported_snapshot_types
            .iter()
            .any(|item| item == horizon.as_str())
        {
            return Err(ApplicationError::Validation(format!(
                "规则包 {} 不支持正式时点 {}",
                decision.package_display_name,
                horizon.as_str()
            )));
        }
    }

    let requested_fact_keys = validate_requested_fact_keys(command.requested_fact_keys)?;
    let research_schema = store
        .read_schema(RESEARCH_SCHEMA_KEY, RESEARCH_SCHEMA_VERSION)
        .await?;
    let snapshot_schema = store
        .read_schema(SNAPSHOT_SCHEMA_KEY, SNAPSHOT_SCHEMA_VERSION)
        .await?;
    Ok(PreparedPlan {
        context,
        decision,
        requested_fact_keys,
        research_schema,
        snapshot_schema,
    })
}
