use super::prepare::PreparedPlan;
use crate::{ApplicationError, ApplicationResult};
use chrono::{DateTime, Duration, Utc};
use football_domain::{
    P4FreezeTaskDraft, P4FreezeTaskState, P4Horizon, P4_FREEZE_GRACE_MINUTES,
    P4_ORCHESTRATION_PLANNER_VERSION, P4_RESEARCH_LEAD_MINUTES,
};
use serde_json::json;
use uuid::Uuid;

pub(super) fn horizon_identity(
    plan: &PreparedPlan,
    horizon: P4Horizon,
) -> ApplicationResult<(DateTime<Utc>, String)> {
    let data_cutoff_at = horizon
        .data_cutoff_at(plan.context.kickoff_at)
        .ok_or_else(|| {
            ApplicationError::Validation(format!("{}不是P4正式时点", horizon.as_str()))
        })?;
    let idempotency_key = format!(
        "p4-freeze:{}:{}:{}:{}:{}:{}",
        plan.context.match_id,
        plan.decision.model_version_id,
        plan.decision.parameter_set_id,
        plan.decision.competition_profile_id,
        horizon.as_str(),
        data_cutoff_at.timestamp()
    );
    Ok((data_cutoff_at, idempotency_key))
}

pub(super) fn task_draft(
    plan: &PreparedPlan,
    horizon: P4Horizon,
    data_cutoff_at: DateTime<Utc>,
    idempotency_key: String,
    now: DateTime<Utc>,
    trace_id: Uuid,
) -> P4FreezeTaskDraft {
    let state = if data_cutoff_at <= now {
        P4FreezeTaskState::Missed
    } else {
        P4FreezeTaskState::Planned
    };
    P4FreezeTaskDraft {
        match_id: plan.context.match_id,
        match_key: plan.context.match_key.clone(),
        horizon,
        kickoff_at: plan.context.kickoff_at,
        data_cutoff_at,
        research_due_at: data_cutoff_at - Duration::minutes(P4_RESEARCH_LEAD_MINUTES),
        freeze_deadline_at: data_cutoff_at + Duration::minutes(P4_FREEZE_GRACE_MINUTES),
        rule_package_id: plan.decision.rule_package_id,
        model_version_id: plan.decision.model_version_id,
        parameter_set_id: plan.decision.parameter_set_id,
        competition_profile_id: plan.decision.competition_profile_id,
        research_schema_version_id: plan.research_schema.id,
        snapshot_schema_version_id: plan.snapshot_schema.id,
        requested_fact_keys: plan.requested_fact_keys.clone(),
        trace_id,
        state,
        idempotency_key,
        metadata: json!({
            "planner_version": P4_ORCHESTRATION_PLANNER_VERSION,
            "package_key": plan.decision.package_key,
            "package_version": plan.decision.package_version,
            "home_team_name": plan.context.home_team_name,
            "away_team_name": plan.context.away_team_name,
            "research_lead_minutes": P4_RESEARCH_LEAD_MINUTES,
            "freeze_grace_minutes": P4_FREEZE_GRACE_MINUTES,
        }),
    }
}
