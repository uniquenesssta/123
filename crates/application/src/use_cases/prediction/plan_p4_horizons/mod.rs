use super::shared::p4_planning::{
    horizon_priority, is_p4_model, validate_existing_task_identity, validate_requested_fact_keys,
};
use super::P4PlanningAccess;
use crate::built_in_artifacts::{
    P4_RESEARCH_SCHEMA_ARTIFACT_VERSION as RESEARCH_SCHEMA_VERSION,
    P4_RESEARCH_SCHEMA_KEY as RESEARCH_SCHEMA_KEY,
    P4_SNAPSHOT_SCHEMA_ARTIFACT_VERSION as SNAPSHOT_SCHEMA_VERSION,
    P4_SNAPSHOT_SCHEMA_KEY as SNAPSHOT_SCHEMA_KEY,
};
use crate::{ApplicationError, ApplicationResult};
use chrono::{Duration, Utc};
use football_domain::{
    EnqueueJobDraft, P4FreezeTaskDraft, P4FreezeTaskRecord, P4FreezeTaskState,
    P4FreezeTaskTransition, P4Horizon, PlanP4HorizonsCommand, RouteRequest,
    P4_FREEZE_GRACE_MINUTES, P4_ORCHESTRATION_PLANNER_VERSION, P4_RESEARCH_LEAD_MINUTES,
};
use serde_json::{json, Value};
use uuid::Uuid;

const P4_RESEARCH_JOB: &str = "p4_horizon_research";

pub(crate) async fn execute<P: P4PlanningAccess + ?Sized>(
    port: &P,
    command: PlanP4HorizonsCommand,
) -> ApplicationResult<Vec<P4FreezeTaskRecord>> {
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
    let now = Utc::now();
    let mut tasks = Vec::with_capacity(P4Horizon::CANONICAL.len());
    for horizon in P4Horizon::CANONICAL {
        let data_cutoff_at = horizon.data_cutoff_at(context.kickoff_at).ok_or_else(|| {
            ApplicationError::Validation(format!("{}不是P4正式时点", horizon.as_str()))
        })?;
        let idempotency_key = format!(
            "p4-freeze:{}:{}:{}:{}:{}:{}",
            context.match_id,
            decision.model_version_id,
            decision.parameter_set_id,
            decision.competition_profile_id,
            horizon.as_str(),
            data_cutoff_at.timestamp()
        );
        if let Some(existing) = store
            .find_freeze_task_by_idempotency(&idempotency_key)
            .await?
        {
            validate_existing_task_identity(
                &existing,
                &decision,
                research_schema.id,
                snapshot_schema.id,
                &requested_fact_keys,
            )?;
            tasks.push(existing);
            continue;
        }
        let state = if data_cutoff_at <= now {
            P4FreezeTaskState::Missed
        } else {
            P4FreezeTaskState::Planned
        };
        let task = store
            .create_freeze_task(&P4FreezeTaskDraft {
                match_id: context.match_id,
                match_key: context.match_key.clone(),
                horizon,
                kickoff_at: context.kickoff_at,
                data_cutoff_at,
                research_due_at: data_cutoff_at - Duration::minutes(P4_RESEARCH_LEAD_MINUTES),
                freeze_deadline_at: data_cutoff_at + Duration::minutes(P4_FREEZE_GRACE_MINUTES),
                rule_package_id: decision.rule_package_id,
                model_version_id: decision.model_version_id,
                parameter_set_id: decision.parameter_set_id,
                competition_profile_id: decision.competition_profile_id,
                research_schema_version_id: research_schema.id,
                snapshot_schema_version_id: snapshot_schema.id,
                requested_fact_keys: requested_fact_keys.clone(),
                trace_id: Uuid::new_v4(),
                state,
                idempotency_key,
                metadata: json!({
                    "planner_version": P4_ORCHESTRATION_PLANNER_VERSION,
                    "package_key": decision.package_key,
                    "package_version": decision.package_version,
                    "home_team_name": context.home_team_name,
                    "away_team_name": context.away_team_name,
                    "research_lead_minutes": P4_RESEARCH_LEAD_MINUTES,
                    "freeze_grace_minutes": P4_FREEZE_GRACE_MINUTES,
                }),
            })
            .await?;
        if state == P4FreezeTaskState::Missed {
            tasks.push(task);
            continue;
        }
        let research_job = store
            .enqueue(&EnqueueJobDraft {
                job_type: P4_RESEARCH_JOB.to_string(),
                payload: json!({"task_id": task.id}),
                idempotency_key: Some(format!("p4-research-job:{}", task.id)),
                available_at: Some(task.research_due_at),
                priority: horizon_priority(horizon),
                max_attempts: 3,
            })
            .await?;
        let queued = store
            .transition_freeze_task(
                task.id,
                &P4FreezeTaskTransition {
                    task_id: task.id,
                    expected_state: P4FreezeTaskState::Planned,
                    next_state: P4FreezeTaskState::ResearchQueued,
                    reason: "研究任务已进入预约后台队列".to_string(),
                    blockers: Value::Null,
                    payload: json!({
                        "job_id": research_job.id,
                        "available_at": research_job.available_at,
                    }),
                    research_run_id: None,
                    research_job_id: Some(research_job.id),
                    freeze_job_id: None,
                    snapshot_id: None,
                },
            )
            .await?;
        tasks.push(queued);
    }
    Ok(tasks)
}
