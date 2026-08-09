use super::{ApplicationError, ApplicationResult, ApplicationService, OpenAiResearchCommand};
use crate::use_cases::prediction::shared::p4_planning::horizon_priority;
use crate::PersistenceStore;
use chrono::Utc;
use football_domain::{
    EnqueueJobDraft, P4FreezeTaskRecord, P4FreezeTaskState, P4FreezeTaskTransition,
    ResearchRunDraft, ResearchRunStatus, P4_ORCHESTRATION_PLANNER_VERSION,
};
use football_research_gateway::{CancellationToken, GatewayOperation};
use serde::Deserialize;
use serde_json::{json, Value};
use std::sync::{atomic::Ordering, Arc};
use tokio::time::{sleep, Duration as TokioDuration};
use uuid::Uuid;

const P4_RESEARCH_JOB: &str = "p4_horizon_research";
const P4_FREEZE_JOB: &str = "p4_horizon_freeze";
const P4_WORKER_POLL_SECONDS: u64 = 30;

#[derive(Debug, Deserialize)]
struct OrchestrationJobPayload {
    task_id: Uuid,
}

impl ApplicationService {
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
        store: &PersistenceStore,
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
            P4_FREEZE_JOB => self.execute_p4_freeze_task(payload.task_id, job_id).await,
            other => Err(ApplicationError::Validation(format!(
                "P4编排器不支持后台任务：{other}"
            ))),
        }
    }

    async fn execute_p4_research_task(
        &self,
        store: &PersistenceStore,
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
    store: &PersistenceStore,
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
    store: &PersistenceStore,
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
    store: &PersistenceStore,
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
    store: &PersistenceStore,
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::use_cases::prediction::shared::p4_planning::{canonical_fact_keys, is_p4_model};
    use football_domain::P4Horizon;
    use std::collections::BTreeSet;

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
