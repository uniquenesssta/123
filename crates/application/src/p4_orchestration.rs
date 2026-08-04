use super::{
    ApplicationError, ApplicationResult, ApplicationService, OpenAiResearchCommand,
    PredictionCommand,
};
use crate::model_shell::P4_MODEL_ID;
use chrono::{Duration, Utc};
use football_domain::{
    EnqueueJobDraft, EvidenceVerificationState, P4FreezeReadiness, P4FreezeTaskDraft,
    P4FreezeTaskEventRecord, P4FreezeTaskRecord, P4FreezeTaskState, P4FreezeTaskTransition,
    P4Horizon, P4RoutedFact, PlanP4HorizonsCommand, PrematchSnapshotDraft, ResearchRunDraft,
    ResearchRunStatus, SnapshotFeatureDraft, SnapshotProbabilityDraft, SnapshotSourceKind,
    P4_FREEZE_GRACE_MINUTES, P4_ORCHESTRATION_PLANNER_VERSION, P4_RESEARCH_LEAD_MINUTES,
};
use football_persistence_postgres::PostgresStore;
use football_research_gateway::{CancellationToken, GatewayOperation};
use serde::Deserialize;
use serde_json::{json, Map, Value};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::{atomic::Ordering, Arc};
use tokio::time::{sleep, Duration as TokioDuration};
use uuid::Uuid;

const RESEARCH_SCHEMA_KEY: &str = "p4-openai-research-output";
const RESEARCH_SCHEMA_VERSION: &str = "2.0.0";
const SNAPSHOT_SCHEMA_KEY: &str = "p4-prematch-snapshot";
const SNAPSHOT_SCHEMA_VERSION: &str = "1.0.0";
const P4_RESEARCH_JOB: &str = "p4_horizon_research";
const P4_FREEZE_JOB: &str = "p4_horizon_freeze";
const P4_WORKER_POLL_SECONDS: u64 = 30;

#[derive(Debug, Deserialize)]
struct OrchestrationJobPayload {
    task_id: Uuid,
}

impl ApplicationService {
    pub async fn plan_p4_horizons(
        &self,
        command: PlanP4HorizonsCommand,
    ) -> ApplicationResult<Vec<P4FreezeTaskRecord>> {
        let store = self.active_store().await?;
        let context = store.p4_planning_match_context(command.match_id).await?;
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
            .read_schema_version_by_key(RESEARCH_SCHEMA_KEY, RESEARCH_SCHEMA_VERSION)
            .await?;
        let snapshot_schema = store
            .read_schema_version_by_key(SNAPSHOT_SCHEMA_KEY, SNAPSHOT_SCHEMA_VERSION)
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
                .find_p4_freeze_task_by_idempotency(&idempotency_key)
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
                .create_p4_freeze_task(&P4FreezeTaskDraft {
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
                .enqueue_job(&EnqueueJobDraft {
                    job_type: P4_RESEARCH_JOB.to_string(),
                    payload: json!({"task_id": task.id}),
                    idempotency_key: Some(format!("p4-research-job:{}", task.id)),
                    available_at: Some(task.research_due_at),
                    priority: horizon_priority(horizon),
                    max_attempts: 3,
                })
                .await?;
            let queued = store
                .transition_p4_freeze_task(&P4FreezeTaskTransition {
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
                })
                .await?;
            tasks.push(queued);
        }
        Ok(tasks)
    }

    pub async fn list_p4_freeze_tasks(
        &self,
        match_id: Option<Uuid>,
        limit: u32,
    ) -> ApplicationResult<Vec<P4FreezeTaskRecord>> {
        Ok(self
            .active_store()
            .await?
            .list_p4_freeze_tasks(match_id, limit)
            .await?)
    }

    pub async fn read_p4_freeze_task(
        &self,
        task_id: Uuid,
    ) -> ApplicationResult<P4FreezeTaskRecord> {
        Ok(self
            .active_store()
            .await?
            .read_p4_freeze_task(task_id)
            .await?)
    }

    pub async fn list_p4_freeze_task_events(
        &self,
        task_id: Uuid,
    ) -> ApplicationResult<Vec<P4FreezeTaskEventRecord>> {
        Ok(self
            .active_store()
            .await?
            .list_p4_freeze_task_events(task_id)
            .await?)
    }

    pub async fn p4_freeze_readiness(&self, task_id: Uuid) -> ApplicationResult<P4FreezeReadiness> {
        Ok(self
            .active_store()
            .await?
            .p4_freeze_readiness(task_id)
            .await?)
    }

    pub async fn process_next_p4_orchestration_job(
        self: &Arc<Self>,
    ) -> ApplicationResult<Option<Value>> {
        let store = self.active_store().await?;
        let Some(job) = store
            .claim_next_job_by_types(&[P4_RESEARCH_JOB, P4_FREEZE_JOB])
            .await?
        else {
            return Ok(None);
        };
        let result = self
            .execute_p4_orchestration_job(&store, &job.job_type, &job.payload, job.id)
            .await;
        match result {
            Ok(value) => {
                store.complete_job(job.id, value.clone()).await?;
                Ok(Some(value))
            }
            Err(error) => {
                if job.attempts >= job.max_attempts {
                    if let Ok(payload) =
                        serde_json::from_value::<OrchestrationJobPayload>(job.payload.clone())
                    {
                        if let Ok(task) = store.read_p4_freeze_task(payload.task_id).await {
                            if !task.state.is_terminal()
                                && task.state.can_transition_to(P4FreezeTaskState::Failed)
                            {
                                let _ = store
                                    .transition_p4_freeze_task(&P4FreezeTaskTransition {
                                        task_id: task.id,
                                        expected_state: task.state,
                                        next_state: P4FreezeTaskState::Failed,
                                        reason: "P4编排任务达到最大尝试次数".to_string(),
                                        blockers: json!([error.to_string()]),
                                        payload: json!({
                                            "job_id": job.id,
                                            "attempts": job.attempts,
                                            "max_attempts": job.max_attempts,
                                        }),
                                        research_run_id: None,
                                        research_job_id: None,
                                        freeze_job_id: None,
                                        snapshot_id: None,
                                    })
                                    .await;
                            }
                        }
                    }
                }
                store.fail_job(job.id, &error.to_string()).await?;
                Err(error)
            }
        }
    }

    async fn execute_p4_orchestration_job(
        &self,
        store: &PostgresStore,
        job_type: &str,
        payload: &Value,
        job_id: Uuid,
    ) -> ApplicationResult<Value> {
        let payload: OrchestrationJobPayload = serde_json::from_value(payload.clone())?;
        match job_type {
            P4_RESEARCH_JOB => {
                self.execute_p4_research_task(store, payload.task_id, job_id)
                    .await
            }
            P4_FREEZE_JOB => {
                self.execute_p4_freeze_task(store, payload.task_id, job_id)
                    .await
            }
            other => Err(ApplicationError::Validation(format!(
                "P4编排器不支持后台任务：{other}"
            ))),
        }
    }

    async fn execute_p4_research_task(
        &self,
        store: &PostgresStore,
        task_id: Uuid,
        job_id: Uuid,
    ) -> ApplicationResult<Value> {
        let mut task = store.read_p4_freeze_task(task_id).await?;
        if task.state.is_terminal() {
            return Ok(json!({"task_id": task.id, "state": task.state, "noop": true}));
        }
        if matches!(
            task.state,
            P4FreezeTaskState::ReadyToFreeze | P4FreezeTaskState::Freezing
        ) {
            return Ok(json!({"task_id": task.id, "state": task.state, "noop": true}));
        }
        if task.state == P4FreezeTaskState::ResearchSucceeded {
            let ready = finalize_successful_research(store, &task).await?;
            return Ok(json!({"task_id": ready.id, "state": ready.state}));
        }
        if task.state == P4FreezeTaskState::ResearchPartial {
            let blocked = block_partial_research(store, &task).await?;
            return Ok(json!({"task_id": blocked.id, "state": blocked.state}));
        }
        let now = Utc::now();
        if now >= task.data_cutoff_at {
            task = transition_missed(
                store,
                &task,
                "研究任务未能在数据截止时间前开始",
                json!({"job_id": job_id, "now": now}),
            )
            .await?;
            return Ok(json!({"task_id": task.id, "state": task.state}));
        }
        if task.state == P4FreezeTaskState::Planned {
            task = store
                .transition_p4_freeze_task(&P4FreezeTaskTransition {
                    task_id: task.id,
                    expected_state: P4FreezeTaskState::Planned,
                    next_state: P4FreezeTaskState::ResearchQueued,
                    reason: "恢复已创建但尚未登记队列状态的研究任务".to_string(),
                    blockers: Value::Null,
                    payload: json!({"job_id": job_id, "recovered": true}),
                    research_run_id: None,
                    research_job_id: Some(job_id),
                    freeze_job_id: None,
                    snapshot_id: None,
                })
                .await?;
        }
        if task.state == P4FreezeTaskState::ResearchQueued {
            let research_run = store
                .create_research_run(&ResearchRunDraft {
                    match_id: task.match_id,
                    horizon: task.horizon,
                    data_cutoff_at: task.data_cutoff_at,
                    trace_id: task.trace_id,
                    idempotency_key: format!("p4-research-run:{}", task.id),
                    planner_version: Some(P4_ORCHESTRATION_PLANNER_VERSION.to_string()),
                    prompt_version_id: None,
                    schema_version_id: task.research_schema_version_id,
                    request_payload: json!({
                        "requested_fact_keys": task.requested_fact_keys.clone(),
                        "orchestration_task_id": task.id,
                    }),
                    metadata: json!({
                        "stage": "F",
                        "job_id": job_id,
                    }),
                })
                .await?;
            task = store
                .transition_p4_freeze_task(&P4FreezeTaskTransition {
                    task_id: task.id,
                    expected_state: P4FreezeTaskState::ResearchQueued,
                    next_state: P4FreezeTaskState::ResearchRunning,
                    reason: "联网事实研究开始".to_string(),
                    blockers: Value::Null,
                    payload: json!({"job_id": job_id, "research_run_id": research_run.id}),
                    research_run_id: Some(research_run.id),
                    research_job_id: None,
                    freeze_job_id: None,
                    snapshot_id: None,
                })
                .await?;
        }
        if task.state != P4FreezeTaskState::ResearchRunning {
            return Err(ApplicationError::Validation(format!(
                "研究任务状态不是RESEARCH_RUNNING：{}",
                task.state.as_str()
            )));
        }
        let research_run_id = task.research_run_id.ok_or_else(|| {
            ApplicationError::Validation("RESEARCH_RUNNING任务缺少research_run_id".to_string())
        })?;
        let existing_run = store.read_research_run(research_run_id).await?;
        match existing_run.status {
            ResearchRunStatus::Cancelled => {
                let cancelled = store
                    .transition_p4_freeze_task(&P4FreezeTaskTransition {
                        task_id: task.id,
                        expected_state: P4FreezeTaskState::ResearchRunning,
                        next_state: P4FreezeTaskState::Cancelled,
                        reason: "恢复任务时发现研究任务已取消".to_string(),
                        blockers: json!(["research_run_cancelled"]),
                        payload: json!({"research_run_id": research_run_id, "job_id": job_id}),
                        research_run_id: None,
                        research_job_id: None,
                        freeze_job_id: None,
                        snapshot_id: None,
                    })
                    .await?;
                return Ok(json!({"task_id": cancelled.id, "state": cancelled.state}));
            }
            ResearchRunStatus::Succeeded | ResearchRunStatus::Partial => {}
            ResearchRunStatus::Planned | ResearchRunStatus::Running | ResearchRunStatus::Failed => {
                let dynamic_context = research_dynamic_context(store, &task).await?;
                if let Err(error) = self
                    .execute_p4_openai_research(
                        OpenAiResearchCommand {
                            research_run_id,
                            trace_id: task.trace_id,
                            match_key: task.match_key.clone(),
                            data_cutoff_at: task.data_cutoff_at,
                            operation: GatewayOperation::Research,
                            dynamic_context,
                            requested_fact_keys: task.requested_fact_keys.clone(),
                        },
                        CancellationToken::new(),
                    )
                    .await
                {
                    let failed_run = store.read_research_run(research_run_id).await?;
                    if failed_run.status == ResearchRunStatus::Partial {
                        let partial = store
                            .transition_p4_freeze_task(&P4FreezeTaskTransition {
                                task_id: task.id,
                                expected_state: P4FreezeTaskState::ResearchRunning,
                                next_state: P4FreezeTaskState::ResearchPartial,
                                reason: "联网结果已保存，但事实流水线存在阻断".to_string(),
                                blockers: json!([error.to_string()]),
                                payload: json!({"job_id": job_id, "research_run_id": research_run_id}),
                                research_run_id: None,
                                research_job_id: None,
                                freeze_job_id: None,
                                snapshot_id: None,
                            })
                            .await?;
                        let readiness = store.p4_freeze_readiness(partial.id).await?;
                        let blocked = store
                            .transition_p4_freeze_task(&P4FreezeTaskTransition {
                                task_id: partial.id,
                                expected_state: P4FreezeTaskState::ResearchPartial,
                                next_state: P4FreezeTaskState::Blocked,
                                reason: "部分研究结果不得进入正式冻结队列".to_string(),
                                blockers: serde_json::to_value(&readiness.blockers)?,
                                payload: serde_json::to_value(&readiness)?,
                                research_run_id: None,
                                research_job_id: None,
                                freeze_job_id: None,
                                snapshot_id: None,
                            })
                            .await?;
                        return Ok(json!({"task_id": blocked.id, "state": blocked.state}));
                    }
                    return Err(error);
                }
            }
        }
        let run = store.read_research_run(research_run_id).await?;
        match run.status {
            ResearchRunStatus::Succeeded => {
                task = store
                    .transition_p4_freeze_task(&P4FreezeTaskTransition {
                        task_id: task.id,
                        expected_state: P4FreezeTaskState::ResearchRunning,
                        next_state: P4FreezeTaskState::ResearchSucceeded,
                        reason: "联网事实研究与证据路由成功".to_string(),
                        blockers: Value::Null,
                        payload: json!({"research_run_id": research_run_id}),
                        research_run_id: None,
                        research_job_id: None,
                        freeze_job_id: None,
                        snapshot_id: None,
                    })
                    .await?;
                let ready = finalize_successful_research(store, &task).await?;
                Ok(json!({"task_id": ready.id, "state": ready.state}))
            }
            ResearchRunStatus::Partial => {
                task = store
                    .transition_p4_freeze_task(&P4FreezeTaskTransition {
                        task_id: task.id,
                        expected_state: P4FreezeTaskState::ResearchRunning,
                        next_state: P4FreezeTaskState::ResearchPartial,
                        reason: "联网事实研究存在实体、时间、来源或冲突阻断".to_string(),
                        blockers: json!(["research_run_partial"]),
                        payload: json!({"research_run_id": research_run_id}),
                        research_run_id: None,
                        research_job_id: None,
                        freeze_job_id: None,
                        snapshot_id: None,
                    })
                    .await?;
                let blocked = block_partial_research(store, &task).await?;
                Ok(json!({"task_id": blocked.id, "state": blocked.state}))
            }
            status => Err(ApplicationError::Validation(format!(
                "研究任务未进入终态：{}",
                status.as_str()
            ))),
        }
    }

    async fn execute_p4_freeze_task(
        &self,
        store: &PostgresStore,
        task_id: Uuid,
        job_id: Uuid,
    ) -> ApplicationResult<Value> {
        let mut task = store.read_p4_freeze_task(task_id).await?;
        if task.state.is_terminal() {
            return Ok(json!({"task_id": task.id, "state": task.state, "noop": true}));
        }
        if let Some(snapshot_id) = store.find_frozen_p4_snapshot_id(&task).await? {
            if task.state != P4FreezeTaskState::Freezing {
                return Err(ApplicationError::Validation(format!(
                    "已存在不可变快照，但任务状态不是FREEZING：{}",
                    task.state.as_str()
                )));
            }
            let frozen = store
                .transition_p4_freeze_task(&P4FreezeTaskTransition {
                    task_id: task.id,
                    expected_state: P4FreezeTaskState::Freezing,
                    next_state: P4FreezeTaskState::Frozen,
                    reason: "恢复已写入但尚未登记FROZEN状态的不可变快照".to_string(),
                    blockers: json!([]),
                    payload: json!({
                        "snapshot_id": snapshot_id,
                        "job_id": job_id,
                        "recovered": true,
                    }),
                    research_run_id: None,
                    research_job_id: None,
                    freeze_job_id: None,
                    snapshot_id: Some(snapshot_id),
                })
                .await?;
            return Ok(json!({
                "task_id": frozen.id,
                "state": frozen.state,
                "snapshot_id": snapshot_id,
                "recovered": true,
            }));
        }
        let now = Utc::now();
        if now < task.data_cutoff_at {
            return Err(ApplicationError::Validation(format!(
                "冻结任务被提前领取：截止时间 {}",
                task.data_cutoff_at.to_rfc3339()
            )));
        }
        if now > task.freeze_deadline_at {
            task = transition_missed(
                store,
                &task,
                "超过正式冻结宽限窗口，禁止用更晚事实回填",
                json!({"job_id": job_id, "now": now}),
            )
            .await?;
            return Ok(json!({"task_id": task.id, "state": task.state}));
        }
        let readiness = store.p4_freeze_readiness(task.id).await?;
        if !readiness.ready {
            let blocked = store
                .transition_p4_freeze_task(&P4FreezeTaskTransition {
                    task_id: task.id,
                    expected_state: task.state,
                    next_state: P4FreezeTaskState::Blocked,
                    reason: "冻结前复核未通过READY_TO_FREEZE门禁".to_string(),
                    blockers: serde_json::to_value(&readiness.blockers)?,
                    payload: serde_json::to_value(&readiness)?,
                    research_run_id: None,
                    research_job_id: None,
                    freeze_job_id: None,
                    snapshot_id: None,
                })
                .await?;
            return Ok(json!({"task_id": blocked.id, "state": blocked.state}));
        }
        if task.state == P4FreezeTaskState::ResearchSucceeded {
            task = store
                .transition_p4_freeze_task(&P4FreezeTaskTransition {
                    task_id: task.id,
                    expected_state: P4FreezeTaskState::ResearchSucceeded,
                    next_state: P4FreezeTaskState::ReadyToFreeze,
                    reason: "恢复已创建冻结Job但尚未登记READY_TO_FREEZE的任务".to_string(),
                    blockers: json!([]),
                    payload: json!({"job_id": job_id, "recovered": true}),
                    research_run_id: None,
                    research_job_id: None,
                    freeze_job_id: Some(job_id),
                    snapshot_id: None,
                })
                .await?;
        }
        if task.state == P4FreezeTaskState::ReadyToFreeze {
            task = store
                .transition_p4_freeze_task(&P4FreezeTaskTransition {
                    task_id: task.id,
                    expected_state: P4FreezeTaskState::ReadyToFreeze,
                    next_state: P4FreezeTaskState::Freezing,
                    reason: "到达正式数据截止时点，开始生成确定性P4快照".to_string(),
                    blockers: Value::Null,
                    payload: json!({"job_id": job_id, "frozen_at": now}),
                    research_run_id: None,
                    research_job_id: None,
                    freeze_job_id: None,
                    snapshot_id: None,
                })
                .await?;
        }
        if task.state != P4FreezeTaskState::Freezing {
            return Err(ApplicationError::Validation(format!(
                "冻结任务状态不是FREEZING：{}",
                task.state.as_str()
            )));
        }
        let routed_facts = store.p4_routed_facts(task.id).await?;
        let prepared = store
            .prepare_match_prediction_input(task.match_id, task.horizon.as_str(), "p4")
            .await?;
        let mut match_input = prepared.match_input;
        attach_orchestration_input(&mut match_input, &task, &readiness, &routed_facts)?;
        let execution = self
            .execute_prediction(PredictionCommand {
                match_input: match_input.clone(),
                snapshot_type: task.horizon.as_str().to_string(),
                competition_id: prepared.match_record.competition_id,
                season_id: prepared.match_record.season_id,
                stage_id: prepared.match_record.stage_id,
                competition_kind: prepared.competition_kind,
                model_family: "p4".to_string(),
                explicit_rule_package_id: Some(task.rule_package_id),
            })
            .await?;
        validate_pinned_route(&task, &execution.route)?;
        let probabilities = snapshot_probabilities(&execution.output.payload)?;
        let features = snapshot_features(&task, &prepared.data_quality, &readiness, &routed_facts)?;
        let database_quality = match_input
            .get("feature_quality_score")
            .and_then(Value::as_f64)
            .unwrap_or(0.0);
        let route_quality = (readiness.routed_fact_count as f64
            / readiness.requested_fact_count.max(1) as f64)
            .clamp(0.0, 1.0);
        let quality_score = ((database_quality + route_quality) / 2.0).clamp(0.0, 1.0);
        let snapshot_schema = store
            .read_schema_version_by_key(SNAPSHOT_SCHEMA_KEY, SNAPSHOT_SCHEMA_VERSION)
            .await?;
        if snapshot_schema.id != task.snapshot_schema_version_id {
            return Err(ApplicationError::Validation(
                "冻结任务锁定的快照Schema版本发生变化".to_string(),
            ));
        }
        let frozen_at = Utc::now();
        if frozen_at > task.freeze_deadline_at {
            let missed = transition_missed(
                store,
                &task,
                "P4执行完成时已超过正式冻结宽限窗口，拒绝写入快照",
                json!({"job_id": job_id, "now": frozen_at}),
            )
            .await?;
            return Ok(json!({"task_id": missed.id, "state": missed.state}));
        }
        let snapshot = store
            .freeze_prematch_snapshot(&PrematchSnapshotDraft {
                match_id: task.match_id,
                match_key: task.match_key.clone(),
                horizon: task.horizon,
                data_cutoff_at: task.data_cutoff_at,
                frozen_at,
                model_version_id: task.model_version_id,
                parameter_set_id: task.parameter_set_id,
                competition_profile_id: task.competition_profile_id,
                research_run_id: task.research_run_id,
                schema_version_id: task.snapshot_schema_version_id,
                schema_version: SNAPSHOT_SCHEMA_VERSION.to_string(),
                trace_id: task.trace_id,
                idempotency_key: format!("p4-prematch-snapshot:{}", task.id),
                source_kind: SnapshotSourceKind::Real,
                quality_score,
                input_payload: json!({
                    "model_run_id": execution.run_id,
                    "match_input": match_input,
                    "p4_output": execution.output.payload,
                }),
                features,
                probabilities,
                metadata: json!({
                    "orchestration_contract": football_domain::P4_ORCHESTRATION_CONTRACT_VERSION,
                    "orchestration_task_id": task.id,
                    "research_run_id": task.research_run_id,
                    "rule_package_id": task.rule_package_id,
                    "model_run_id": execution.run_id,
                    "readiness": readiness,
                }),
            })
            .await?;
        let frozen = store
            .transition_p4_freeze_task(&P4FreezeTaskTransition {
                task_id: task.id,
                expected_state: P4FreezeTaskState::Freezing,
                next_state: P4FreezeTaskState::Frozen,
                reason: "31字段与四链概率已写入不可变正式快照".to_string(),
                blockers: json!([]),
                payload: json!({
                    "snapshot_id": snapshot.id,
                    "model_run_id": execution.run_id,
                    "quality_score": quality_score,
                }),
                research_run_id: None,
                research_job_id: None,
                freeze_job_id: None,
                snapshot_id: Some(snapshot.id),
            })
            .await?;
        Ok(json!({
            "task_id": frozen.id,
            "state": frozen.state,
            "snapshot_id": snapshot.id,
            "model_run_id": execution.run_id,
        }))
    }
}

pub(crate) fn spawn_p4_orchestration_worker(service: Arc<ApplicationService>) {
    if service.p4_worker_running.swap(true, Ordering::SeqCst) {
        return;
    }
    tokio::spawn(async move {
        loop {
            match service.process_next_p4_orchestration_job().await {
                Ok(Some(_)) => continue,
                Ok(None) => sleep(TokioDuration::from_secs(P4_WORKER_POLL_SECONDS)).await,
                Err(ApplicationError::DatabaseNotConnected) => break,
                Err(_) => sleep(TokioDuration::from_secs(P4_WORKER_POLL_SECONDS)).await,
            }
        }
        service.p4_worker_running.store(false, Ordering::SeqCst);
    });
}

async fn research_dynamic_context(
    store: &PostgresStore,
    task: &P4FreezeTaskRecord,
) -> ApplicationResult<Value> {
    let context = store.p4_planning_match_context(task.match_id).await?;
    Ok(json!({
        "orchestration_task_id": task.id,
        "match": {
            "match_id": context.match_id,
            "match_key": context.match_key,
            "kickoff_at": context.kickoff_at,
            "home_team": context.home_team_name,
            "away_team": context.away_team_name,
            "competition_id": context.competition_id,
            "season_id": context.season_id,
            "stage_id": context.stage_id,
            "competition_kind": context.competition_kind,
        },
        "horizon": task.horizon.as_str(),
        "data_cutoff_at": task.data_cutoff_at,
        "rules": {
            "facts_only": true,
            "no_external_prediction": true,
            "no_betting_advice": true,
            "missing_facts_must_remain_missing": true,
        }
    }))
}

pub(super) async fn finalize_successful_research(
    store: &PostgresStore,
    task: &P4FreezeTaskRecord,
) -> ApplicationResult<P4FreezeTaskRecord> {
    let readiness = store.p4_freeze_readiness(task.id).await?;
    if !readiness.ready {
        return Ok(store
            .transition_p4_freeze_task(&P4FreezeTaskTransition {
                task_id: task.id,
                expected_state: P4FreezeTaskState::ResearchSucceeded,
                next_state: P4FreezeTaskState::Blocked,
                reason: "研究完成但未通过READY_TO_FREEZE门禁".to_string(),
                blockers: serde_json::to_value(&readiness.blockers)?,
                payload: serde_json::to_value(&readiness)?,
                research_run_id: None,
                research_job_id: None,
                freeze_job_id: None,
                snapshot_id: None,
            })
            .await?);
    }
    let freeze_job = store
        .enqueue_job(&EnqueueJobDraft {
            job_type: P4_FREEZE_JOB.to_string(),
            payload: json!({"task_id": task.id}),
            idempotency_key: Some(format!("p4-freeze-job:{}", task.id)),
            available_at: Some(task.data_cutoff_at),
            priority: horizon_priority(task.horizon) + 100,
            max_attempts: 3,
        })
        .await?;
    Ok(store
        .transition_p4_freeze_task(&P4FreezeTaskTransition {
            task_id: task.id,
            expected_state: P4FreezeTaskState::ResearchSucceeded,
            next_state: P4FreezeTaskState::ReadyToFreeze,
            reason: "所有事实路由通过门禁，等待截止时点自动冻结".to_string(),
            blockers: json!([]),
            payload: json!({
                "readiness": readiness,
                "freeze_job_id": freeze_job.id,
                "available_at": freeze_job.available_at,
            }),
            research_run_id: None,
            research_job_id: None,
            freeze_job_id: Some(freeze_job.id),
            snapshot_id: None,
        })
        .await?)
}

async fn block_partial_research(
    store: &PostgresStore,
    task: &P4FreezeTaskRecord,
) -> ApplicationResult<P4FreezeTaskRecord> {
    let readiness = store.p4_freeze_readiness(task.id).await?;
    Ok(store
        .transition_p4_freeze_task(&P4FreezeTaskTransition {
            task_id: task.id,
            expected_state: P4FreezeTaskState::ResearchPartial,
            next_state: P4FreezeTaskState::Blocked,
            reason: "部分研究结果不得进入正式冻结队列".to_string(),
            blockers: serde_json::to_value(&readiness.blockers)?,
            payload: serde_json::to_value(&readiness)?,
            research_run_id: None,
            research_job_id: None,
            freeze_job_id: None,
            snapshot_id: None,
        })
        .await?)
}

async fn transition_missed(
    store: &PostgresStore,
    task: &P4FreezeTaskRecord,
    reason: &str,
    payload: Value,
) -> ApplicationResult<P4FreezeTaskRecord> {
    let next = P4FreezeTaskState::Missed;
    if !task.state.can_transition_to(next) {
        return Err(ApplicationError::Validation(format!(
            "任务状态 {} 不能转为MISSED",
            task.state.as_str()
        )));
    }
    Ok(store
        .transition_p4_freeze_task(&P4FreezeTaskTransition {
            task_id: task.id,
            expected_state: task.state,
            next_state: next,
            reason: reason.to_string(),
            blockers: json!([reason]),
            payload,
            research_run_id: None,
            research_job_id: None,
            freeze_job_id: None,
            snapshot_id: None,
        })
        .await?)
}

fn validate_requested_fact_keys(requested: Vec<String>) -> ApplicationResult<Vec<String>> {
    let canonical = canonical_fact_keys();
    if requested.is_empty() {
        return Ok(canonical);
    }
    let requested_set = requested
        .into_iter()
        .map(|value| value.trim().to_string())
        .collect::<BTreeSet<_>>();
    let canonical_set = canonical.iter().cloned().collect::<BTreeSet<_>>();
    if requested_set != canonical_set {
        return Err(ApplicationError::Validation(
            "正式P4冻结必须研究路由注册表中的全部29个事实字段；不得以子集生成31字段正式快照"
                .to_string(),
        ));
    }
    Ok(canonical)
}

fn canonical_fact_keys() -> Vec<String> {
    let registry: football_domain::EvidenceRouteRegistry = serde_json::from_str(include_str!(
        "../../../src-tauri/resources/research/public_evidence_routes.json"
    ))
    .expect("内置P4证据路由注册表必须有效");
    registry
        .routes
        .into_iter()
        .map(|route| route.field_key)
        .collect()
}

fn is_p4_model(model_id: &str) -> bool {
    model_id == P4_MODEL_ID || model_id.starts_with("p4_")
}

fn horizon_priority(horizon: P4Horizon) -> i32 {
    match horizon {
        P4Horizon::T24h => 10,
        P4Horizon::T6h => 20,
        P4Horizon::T90m => 30,
        P4Horizon::T1h => 40,
        P4Horizon::LegacyTN => 0,
    }
}

fn attach_orchestration_input(
    match_input: &mut Value,
    task: &P4FreezeTaskRecord,
    readiness: &P4FreezeReadiness,
    routed_facts: &[P4RoutedFact],
) -> ApplicationResult<()> {
    let object = match_input.as_object_mut().ok_or_else(|| {
        ApplicationError::Validation("数据库构建的P4输入不是JSON对象".to_string())
    })?;
    object.insert(
        "p4_orchestration".to_string(),
        json!({
            "contract_version": football_domain::P4_ORCHESTRATION_CONTRACT_VERSION,
            "task_id": task.id,
            "trace_id": task.trace_id,
            "horizon": task.horizon.as_str(),
            "data_cutoff_at": task.data_cutoff_at,
            "readiness": readiness,
            "routed_facts": routed_facts,
            "numeric_transform_policy": "freeze_provenance_without_unversioned_weight_invention"
        }),
    );
    Ok(())
}

fn validate_existing_task_identity(
    task: &P4FreezeTaskRecord,
    route: &football_domain::RouteDecision,
    research_schema_version_id: Uuid,
    snapshot_schema_version_id: Uuid,
    requested_fact_keys: &[String],
) -> ApplicationResult<()> {
    let existing_facts = task
        .requested_fact_keys
        .iter()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    let requested_facts = requested_fact_keys
        .iter()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    if task.rule_package_id != route.rule_package_id
        || task.model_version_id != route.model_version_id
        || task.parameter_set_id != route.parameter_set_id
        || task.competition_profile_id != route.competition_profile_id
        || task.research_schema_version_id != research_schema_version_id
        || task.snapshot_schema_version_id != snapshot_schema_version_id
        || existing_facts != requested_facts
    {
        return Err(ApplicationError::Validation(
            "同一正式队列键已存在，但规则包、模型、参数、赛事Profile、Schema或事实字段与当前规划请求不一致"
                .to_string(),
        ));
    }
    Ok(())
}

fn validate_pinned_route(
    task: &P4FreezeTaskRecord,
    route: &football_domain::RouteDecision,
) -> ApplicationResult<()> {
    if route.rule_package_id != task.rule_package_id
        || route.model_version_id != task.model_version_id
        || route.parameter_set_id != task.parameter_set_id
        || route.competition_profile_id != task.competition_profile_id
    {
        return Err(ApplicationError::Validation(
            "冻结执行时的规则包、模型、参数或赛事Profile与规划时锁定身份不一致".to_string(),
        ));
    }
    Ok(())
}

fn snapshot_features(
    task: &P4FreezeTaskRecord,
    data_quality: &Value,
    readiness: &P4FreezeReadiness,
    routes: &[P4RoutedFact],
) -> ApplicationResult<Vec<SnapshotFeatureDraft>> {
    let mut by_field = BTreeMap::<String, Vec<&P4RoutedFact>>::new();
    for route in routes {
        by_field
            .entry(route.field_key.clone())
            .or_default()
            .push(route);
    }
    let mut features = Vec::with_capacity(31);
    for (index, field_key) in canonical_fact_keys().into_iter().enumerate() {
        let field_routes = by_field.get(&field_key).cloned().unwrap_or_default();
        let mut evidence_ids = field_routes
            .iter()
            .flat_map(|route| route.selected_evidence_ids.iter().copied())
            .collect::<Vec<_>>();
        evidence_ids.sort_unstable();
        evidence_ids.dedup();
        let verification_state = aggregate_verification_state(&field_routes)?;
        let value = if field_routes.is_empty() {
            Value::Null
        } else if field_routes.len() == 1 {
            field_routes[0].selected_value.clone()
        } else {
            Value::Array(
                field_routes
                    .iter()
                    .map(|route| {
                        json!({
                            "route_key": route.route_key,
                            "entity_target": {
                                "module": route.target_module,
                                "slot": route.target_slot,
                            },
                            "value": route.selected_value,
                        })
                    })
                    .collect(),
            )
        };
        features.push(SnapshotFeatureDraft {
            field_order: u8::try_from(index + 1)
                .map_err(|_| ApplicationError::Validation("P4快照字段序号溢出".to_string()))?,
            field_key,
            value,
            verification_state,
            evidence_ids,
            metadata: json!({
                "orchestration_task_id": task.id,
                "route_count": field_routes.len(),
            }),
        });
    }
    features.push(SnapshotFeatureDraft {
        field_order: 30,
        field_key: "database_pre_match_features".to_string(),
        value: data_quality.clone(),
        verification_state: EvidenceVerificationState::NotApplicable,
        evidence_ids: Vec::new(),
        metadata: json!({"source": "PostgreSQL cutoff-aware deterministic preparation"}),
    });
    features.push(SnapshotFeatureDraft {
        field_order: 31,
        field_key: "orchestration_readiness".to_string(),
        value: serde_json::to_value(readiness)?,
        verification_state: EvidenceVerificationState::NotApplicable,
        evidence_ids: Vec::new(),
        metadata: json!({
            "contract_version": football_domain::P4_ORCHESTRATION_CONTRACT_VERSION,
            "state": "READY_TO_FREEZE",
        }),
    });
    Ok(features)
}

fn aggregate_verification_state(
    routes: &[&P4RoutedFact],
) -> ApplicationResult<EvidenceVerificationState> {
    if routes.is_empty() {
        return Ok(EvidenceVerificationState::NotFound);
    }
    let mut states = BTreeSet::new();
    for route in routes {
        states.insert(route.verification_state.as_str());
    }
    if states.contains("CONFLICT") || states.contains("STALE") {
        return Err(ApplicationError::Validation(
            "READY_TO_FREEZE任务仍包含CONFLICT或STALE路由".to_string(),
        ));
    }
    if states.contains("PROBABLE") {
        Ok(EvidenceVerificationState::Probable)
    } else if states.contains("CONFIRMED") {
        Ok(EvidenceVerificationState::Confirmed)
    } else if states.contains("NOT_FOUND") {
        Ok(EvidenceVerificationState::NotFound)
    } else {
        Ok(EvidenceVerificationState::NotApplicable)
    }
}

fn snapshot_probabilities(payload: &Value) -> ApplicationResult<Vec<SnapshotProbabilityDraft>> {
    let matrices = payload
        .get("matrices")
        .and_then(Value::as_object)
        .ok_or_else(|| ApplicationError::Validation("外部模型输出缺少 matrices".to_string()))?;
    if matrices.is_empty() {
        return Err(ApplicationError::Validation(
            "外部模型输出至少需要一条概率矩阵".to_string(),
        ));
    }
    let clean_sheet_home = payload.get("clean_sheet_a").and_then(Value::as_f64);
    let clean_sheet_away = payload.get("clean_sheet_b").and_then(Value::as_f64);
    matrices
        .iter()
        .map(|(chain_key, matrix)| {
            let outcome = matrix
                .get("outcome")
                .and_then(Value::as_object)
                .ok_or_else(|| {
                    ApplicationError::Validation(format!("外部模型矩阵 {chain_key} 缺少 outcome"))
                })?;
            let scorelines = matrix
                .get("scorelines")
                .and_then(Value::as_array)
                .ok_or_else(|| {
                    ApplicationError::Validation(format!(
                        "外部模型矩阵 {chain_key} 缺少 scorelines"
                    ))
                })?;
            let matrix_cell_count = u16::try_from(scorelines.len()).map_err(|_| {
                ApplicationError::Validation(format!(
                    "外部模型矩阵 {chain_key} 的比分单元数量超出支持范围"
                ))
            })?;
            if matrix_cell_count == 0 {
                return Err(ApplicationError::Validation(format!(
                    "外部模型矩阵 {chain_key} 不得为空"
                )));
            }
            let is_formal = matrix
                .get("formal")
                .and_then(Value::as_bool)
                .unwrap_or(chain_key == "full");
            Ok(SnapshotProbabilityDraft {
                chain_key: chain_key.to_string(),
                home_win: required_probability(outcome, "a_win", chain_key)?,
                draw: required_probability(outcome, "draw", chain_key)?,
                away_win: required_probability(outcome, "b_win", chain_key)?,
                btts: matrix.get("btts").and_then(Value::as_f64),
                over_2_5: matrix.get("over_2_5").and_then(Value::as_f64),
                clean_sheet_home: is_formal.then_some(clean_sheet_home).flatten(),
                clean_sheet_away: is_formal.then_some(clean_sheet_away).flatten(),
                matrix_sha256: sha256_value(matrix)?,
                matrix_cell_count,
                metadata: json!({
                    "formal": is_formal,
                    "provider_owned_topology": true
                }),
            })
        })
        .collect()
}

fn required_probability(
    outcome: &Map<String, Value>,
    key: &str,
    chain_key: &str,
) -> ApplicationResult<f64> {
    outcome
        .get(key)
        .and_then(Value::as_f64)
        .filter(|value| value.is_finite() && (0.0..=1.0).contains(value))
        .ok_or_else(|| {
            ApplicationError::Validation(format!("外部模型矩阵 {chain_key} 的概率字段 {key} 无效"))
        })
}

fn sha256_value(value: &Value) -> ApplicationResult<String> {
    let bytes = serde_json::to_vec(value)?;
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    Ok(hex::encode(hasher.finalize()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formal_fact_set_has_twenty_nine_unique_fields() {
        let fields = canonical_fact_keys();
        assert_eq!(fields.len(), 29);
        assert_eq!(fields.iter().collect::<BTreeSet<_>>().len(), 29);
    }

    #[test]
    fn only_p4_models_enter_stage_f() {
        assert!(is_p4_model("p4"));
        assert!(is_p4_model("p4_knockout_90"));
        assert!(!is_p4_model("p7"));
    }

    #[test]
    fn canonical_horizon_priority_increases_toward_kickoff() {
        assert!(horizon_priority(P4Horizon::T24h) < horizon_priority(P4Horizon::T6h));
        assert!(horizon_priority(P4Horizon::T6h) < horizon_priority(P4Horizon::T90m));
        assert!(horizon_priority(P4Horizon::T90m) < horizon_priority(P4Horizon::T1h));
    }
}
