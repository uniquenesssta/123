from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def write(path: str, content: str) -> None:
    target = ROOT / path
    target.parent.mkdir(parents=True, exist_ok=True)
    target.write_text(content, encoding="utf-8", newline="\n")


def replace_once(path: str, old: str, new: str, label: str) -> None:
    target = ROOT / path
    text = target.read_text(encoding="utf-8")
    if text.count(old) != 1:
        raise RuntimeError(f"AT4 anchor mismatch: {label}")
    target.write_text(text.replace(old, new, 1), encoding="utf-8", newline="\n")


def insert_after(path: str, needle: str, line: str) -> None:
    target = ROOT / path
    text = target.read_text(encoding="utf-8")
    if line in text:
        return
    lines = text.splitlines()
    for index, current in enumerate(lines):
        if needle in current:
            lines.insert(index + 1, line)
            target.write_text("\n".join(lines) + "\n", encoding="utf-8", newline="\n")
            return
    raise RuntimeError(f"AT4 README anchor not found: {needle}")


write(
    "crates/application/src/use_cases/research/p4_worker/mod.rs",
    '''use crate::ports::{
    analytics::JobQueuePort,
    prediction::PredictionWorkflowPort,
    research::{ResearchArtifactPort, ResearchGatewayAuditPort},
};
use crate::use_cases::research::fact_pipeline::FactPipelineAccess;
use crate::ApplicationResult;
use football_domain::P4FreezeTaskRecord;
use serde_json::Value;
use uuid::Uuid;

mod context;
mod execution;
mod transitions;

pub(crate) async fn execute(
    workflow: &dyn PredictionWorkflowPort,
    jobs: &dyn JobQueuePort,
    artifacts: &dyn ResearchArtifactPort,
    audit: &dyn ResearchGatewayAuditPort,
    pipeline: &dyn FactPipelineAccess,
    task_id: Uuid,
    job_id: Uuid,
) -> ApplicationResult<Value> {
    execution::execute(workflow, jobs, artifacts, audit, pipeline, task_id, job_id).await
}

pub(crate) async fn finalize_successful_research(
    workflow: &dyn PredictionWorkflowPort,
    jobs: &dyn JobQueuePort,
    task: &P4FreezeTaskRecord,
) -> ApplicationResult<P4FreezeTaskRecord> {
    transitions::finalize_successful_research(workflow, jobs, task).await
}
''',
)

write(
    "crates/application/src/use_cases/research/p4_worker/context.rs",
    '''use crate::ports::prediction::PredictionWorkflowPort;
use crate::ApplicationResult;
use football_domain::P4FreezeTaskRecord;
use serde_json::{json, Value};

pub(super) async fn research_dynamic_context(
    workflow: &dyn PredictionWorkflowPort,
    task: &P4FreezeTaskRecord,
) -> ApplicationResult<Value> {
    let context = workflow.planning_match_context(task.match_id).await?;
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
''',
)

write(
    "crates/application/src/use_cases/research/p4_worker/transitions.rs",
    '''use crate::ports::{analytics::JobQueuePort, prediction::PredictionWorkflowPort};
use crate::use_cases::prediction::shared::p4_planning::horizon_priority;
use crate::{ApplicationError, ApplicationResult};
use football_domain::{
    EnqueueJobDraft, P4FreezeTaskRecord, P4FreezeTaskState, P4FreezeTaskTransition,
};
use serde_json::{json, Value};

const P4_FREEZE_JOB: &str = "p4_horizon_freeze";

pub(super) async fn finalize_successful_research(
    workflow: &dyn PredictionWorkflowPort,
    jobs: &dyn JobQueuePort,
    task: &P4FreezeTaskRecord,
) -> ApplicationResult<P4FreezeTaskRecord> {
    let readiness = workflow.freeze_readiness(task.id).await?;
    if !readiness.ready {
        return Ok(workflow
            .transition_freeze_task(
                task.id,
                &P4FreezeTaskTransition {
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
                },
            )
            .await?);
    }
    let freeze_job = jobs
        .enqueue(&EnqueueJobDraft {
            job_type: P4_FREEZE_JOB.to_string(),
            payload: json!({"task_id": task.id}),
            idempotency_key: Some(format!("p4-freeze-job:{}", task.id)),
            available_at: Some(task.data_cutoff_at),
            priority: horizon_priority(task.horizon) + 100,
            max_attempts: 3,
        })
        .await?;
    Ok(workflow
        .transition_freeze_task(
            task.id,
            &P4FreezeTaskTransition {
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
            },
        )
        .await?)
}

pub(super) async fn block_partial_research(
    workflow: &dyn PredictionWorkflowPort,
    task: &P4FreezeTaskRecord,
) -> ApplicationResult<P4FreezeTaskRecord> {
    let readiness = workflow.freeze_readiness(task.id).await?;
    Ok(workflow
        .transition_freeze_task(
            task.id,
            &P4FreezeTaskTransition {
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
            },
        )
        .await?)
}

pub(super) async fn transition_missed(
    workflow: &dyn PredictionWorkflowPort,
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
    Ok(workflow
        .transition_freeze_task(
            task.id,
            &P4FreezeTaskTransition {
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
            },
        )
        .await?)
}
''',
)

write(
    "crates/application/src/use_cases/research/p4_worker/execution.rs",
    '''use super::{context::research_dynamic_context, transitions::*};
use crate::ports::{
    analytics::JobQueuePort,
    prediction::PredictionWorkflowPort,
    research::{ResearchArtifactPort, ResearchGatewayAuditPort},
};
use crate::use_cases::research::{
    fact_pipeline::FactPipelineAccess,
    openai_gateway::{self, OpenAiResearchCommand},
};
use crate::{ApplicationError, ApplicationResult};
use chrono::Utc;
use football_domain::{
    P4FreezeTaskState, P4FreezeTaskTransition, ResearchRunDraft, ResearchRunStatus,
    P4_ORCHESTRATION_PLANNER_VERSION,
};
use football_research_gateway::{CancellationToken, GatewayOperation};
use serde_json::{json, Value};
use uuid::Uuid;

pub(super) async fn execute(
    workflow: &dyn PredictionWorkflowPort,
    jobs: &dyn JobQueuePort,
    artifacts: &dyn ResearchArtifactPort,
    audit: &dyn ResearchGatewayAuditPort,
    pipeline: &dyn FactPipelineAccess,
    task_id: Uuid,
    job_id: Uuid,
) -> ApplicationResult<Value> {
    let mut task = workflow.read_freeze_task(task_id).await?;
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
        let ready = finalize_successful_research(workflow, jobs, &task).await?;
        return Ok(json!({"task_id": ready.id, "state": ready.state}));
    }
    if task.state == P4FreezeTaskState::ResearchPartial {
        let blocked = block_partial_research(workflow, &task).await?;
        return Ok(json!({"task_id": blocked.id, "state": blocked.state}));
    }
    let now = Utc::now();
    if now >= task.data_cutoff_at {
        task = transition_missed(
            workflow,
            &task,
            "研究任务未能在数据截止时间前开始",
            json!({"job_id": job_id, "now": now}),
        )
        .await?;
        return Ok(json!({"task_id": task.id, "state": task.state}));
    }
    if task.state == P4FreezeTaskState::Planned {
        task = workflow
            .transition_freeze_task(
                task.id,
                &P4FreezeTaskTransition {
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
                },
            )
            .await?;
    }
    if task.state == P4FreezeTaskState::ResearchQueued {
        let research_run = artifacts
            .create_run(&ResearchRunDraft {
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
        task = workflow
            .transition_freeze_task(
                task.id,
                &P4FreezeTaskTransition {
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
                },
            )
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
    let existing_run = artifacts.read_run(research_run_id).await?;
    match existing_run.status {
        ResearchRunStatus::Cancelled => {
            let cancelled = workflow
                .transition_freeze_task(
                    task.id,
                    &P4FreezeTaskTransition {
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
                    },
                )
                .await?;
            return Ok(json!({"task_id": cancelled.id, "state": cancelled.state}));
        }
        ResearchRunStatus::Succeeded | ResearchRunStatus::Partial => {}
        ResearchRunStatus::Planned | ResearchRunStatus::Running | ResearchRunStatus::Failed => {
            let dynamic_context = research_dynamic_context(workflow, &task).await?;
            if let Err(error) = openai_gateway::execute_p4_openai_research(
                artifacts,
                audit,
                pipeline,
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
                let failed_run = artifacts.read_run(research_run_id).await?;
                if failed_run.status == ResearchRunStatus::Partial {
                    let partial = workflow
                        .transition_freeze_task(
                            task.id,
                            &P4FreezeTaskTransition {
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
                            },
                        )
                        .await?;
                    let blocked = block_partial_research(workflow, &partial).await?;
                    return Ok(json!({"task_id": blocked.id, "state": blocked.state}));
                }
                return Err(error);
            }
        }
    }
    let run = artifacts.read_run(research_run_id).await?;
    match run.status {
        ResearchRunStatus::Succeeded => {
            task = workflow
                .transition_freeze_task(
                    task.id,
                    &P4FreezeTaskTransition {
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
                    },
                )
                .await?;
            let ready = finalize_successful_research(workflow, jobs, &task).await?;
            Ok(json!({"task_id": ready.id, "state": ready.state}))
        }
        ResearchRunStatus::Partial => {
            task = workflow
                .transition_freeze_task(
                    task.id,
                    &P4FreezeTaskTransition {
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
                    },
                )
                .await?;
            let blocked = block_partial_research(workflow, &task).await?;
            Ok(json!({"task_id": blocked.id, "state": blocked.state}))
        }
        status => Err(ApplicationError::Validation(format!(
            "研究任务未进入终态：{}",
            status.as_str()
        ))),
    }
}
''',
)

replace_once(
    "crates/application/src/use_cases/research/mod.rs",
    "pub(crate) mod openai_gateway;\n",
    "pub(crate) mod openai_gateway;\npub(crate) mod p4_worker;\n",
    "research use-case module registration",
)

replace_once(
    "crates/application/src/services/research/service.rs",
    '''use crate::ports::research::{
    ResearchArtifactPort, ResearchEvidenceLedgerPort, ResearchGatewayAuditPort,
};
''',
    '''use crate::ports::{
    analytics::JobQueuePort,
    prediction::PredictionWorkflowPort,
    research::{ResearchArtifactPort, ResearchEvidenceLedgerPort, ResearchGatewayAuditPort},
};
''',
    "research service ports",
)
replace_once(
    "crates/application/src/services/research/service.rs",
    '''    ledger,
    openai_gateway::{self, OpenAiResearchCommand},
};
''',
    '''    ledger,
    openai_gateway::{self, OpenAiResearchCommand},
    p4_worker,
};
''',
    "research service p4 worker import",
)
replace_once(
    "crates/application/src/services/research/service.rs",
    '''    PromptVersionDraft, PromptVersionRecord, ResearchRunDraft, ResearchRunEventDraft,
    ResearchRunRecord, SchemaVersionDraft, SchemaVersionRecord,
};
''',
    '''    P4FreezeTaskRecord, PromptVersionDraft, PromptVersionRecord, ResearchRunDraft,
    ResearchRunEventDraft, ResearchRunRecord, SchemaVersionDraft, SchemaVersionRecord,
};
''',
    "research service p4 task type",
)
replace_once(
    "crates/application/src/services/research/service.rs",
    "use football_research_gateway::{CancellationToken, GatewayExecution};\n",
    "use football_research_gateway::{CancellationToken, GatewayExecution};\nuse serde_json::Value;\nuse uuid::Uuid;\n",
    "research service worker value imports",
)

service_path = ROOT / "crates/application/src/services/research/service.rs"
service = service_path.read_text(encoding="utf-8")
service_insert = '''

    pub(crate) async fn execute_p4_research_task(
        &self,
        workflow: &dyn PredictionWorkflowPort,
        jobs: &dyn JobQueuePort,
        artifacts: &dyn ResearchArtifactPort,
        audit: &dyn ResearchGatewayAuditPort,
        pipeline: &dyn FactPipelineAccess,
        task_id: Uuid,
        job_id: Uuid,
    ) -> ApplicationResult<Value> {
        p4_worker::execute(
            workflow, jobs, artifacts, audit, pipeline, task_id, job_id,
        )
        .await
    }

    pub(crate) async fn finalize_successful_research(
        &self,
        workflow: &dyn PredictionWorkflowPort,
        jobs: &dyn JobQueuePort,
        task: &P4FreezeTaskRecord,
    ) -> ApplicationResult<P4FreezeTaskRecord> {
        p4_worker::finalize_successful_research(workflow, jobs, task).await
    }
'''
pos = service.rfind("\n}")
if pos < 0 or "fn execute_p4_research_task" in service:
    raise RuntimeError("AT4 ResearchService insertion anchor mismatch")
service_path.write_text(service[:pos] + service_insert + service[pos:], encoding="utf-8", newline="\n")

replace_once(
    "crates/application/src/services/research/facade.rs",
    '''    PromptVersionDraft, PromptVersionRecord, ResearchRunDraft, ResearchRunEventDraft,
    ResearchRunRecord, SchemaVersionDraft, SchemaVersionRecord,
};
''',
    '''    P4FreezeTaskRecord, PromptVersionDraft, PromptVersionRecord, ResearchRunDraft,
    ResearchRunEventDraft, ResearchRunRecord, SchemaVersionDraft, SchemaVersionRecord,
};
''',
    "research facade p4 task type",
)
replace_once(
    "crates/application/src/services/research/facade.rs",
    "use football_research_gateway::{CancellationToken, GatewayExecution};\n",
    "use football_research_gateway::{CancellationToken, GatewayExecution};\nuse serde_json::Value;\nuse uuid::Uuid;\n",
    "research facade worker imports",
)

facade_path = ROOT / "crates/application/src/services/research/facade.rs"
facade = facade_path.read_text(encoding="utf-8")
facade_insert = '''

    pub(crate) async fn execute_p4_research_task(
        &self,
        task_id: Uuid,
        job_id: Uuid,
    ) -> ApplicationResult<Value> {
        let session = self.research_session().await?;
        self.research
            .execute_p4_research_task(
                &session,
                &session,
                &session,
                &session,
                &session,
                task_id,
                job_id,
            )
            .await
    }

    pub(crate) async fn finalize_p4_research_task(
        &self,
        task: &P4FreezeTaskRecord,
    ) -> ApplicationResult<P4FreezeTaskRecord> {
        let session = self.research_session().await?;
        self.research
            .finalize_successful_research(&session, &session, task)
            .await
    }
'''
pos = facade.rfind("\n}")
if pos < 0 or "fn execute_p4_research_task" in facade:
    raise RuntimeError("AT4 Research facade insertion anchor mismatch")
facade_path.write_text(facade[:pos] + facade_insert + facade[pos:], encoding="utf-8", newline="\n")

write(
    "crates/application/src/p4_orchestration.rs",
    '''use super::{ApplicationError, ApplicationResult, ApplicationService};
use football_domain::{P4FreezeTaskState, P4FreezeTaskTransition};
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
            .execute_p4_orchestration_job(&job.job_type, &job.payload, job.id)
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
        job_type: &str,
        payload: &Value,
        job_id: Uuid,
    ) -> ApplicationResult<Value> {
        let payload: OrchestrationJobPayload = serde_json::from_value(payload.clone())?;
        match job_type {
            P4_RESEARCH_JOB => self.execute_p4_research_task(payload.task_id, job_id).await,
            P4_FREEZE_JOB => self.execute_p4_freeze_task(payload.task_id, job_id).await,
            other => Err(ApplicationError::Validation(format!(
                "P4编排器不支持后台任务：{other}"
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

#[cfg(test)]
mod tests {
    use crate::use_cases::prediction::shared::p4_planning::{
        canonical_fact_keys, horizon_priority, is_p4_model,
    };
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
''',
)

replace_once(
    "crates/application/src/p4_workbench.rs",
    '''use super::{
    p4_orchestration::finalize_successful_research, ApplicationError, ApplicationResult,
    ApplicationService,
};
''',
    '''use super::{ApplicationError, ApplicationResult, ApplicationService};
''',
    "manual conflict legacy finalizer import",
)
replace_once(
    "crates/application/src/p4_workbench.rs",
    "reconcile_p4_task_after_manual_decision(&store, task.id, research_run_id).await?;",
    "reconcile_p4_task_after_manual_decision(self, &store, task.id, research_run_id).await?;",
    "manual conflict idempotent reconciliation call",
)
replace_once(
    "crates/application/src/p4_workbench.rs",
    "reconcile_p4_task_after_manual_decision(&store, task.id, research_run_id).await?;",
    "reconcile_p4_task_after_manual_decision(self, &store, task.id, research_run_id).await?;",
    "manual conflict post-write reconciliation call",
)
replace_once(
    "crates/application/src/p4_workbench.rs",
    '''async fn reconcile_p4_task_after_manual_decision(
    store: &PersistenceStore,
    task_id: Uuid,
    research_run_id: Uuid,
) -> ApplicationResult<()> {
''',
    '''async fn reconcile_p4_task_after_manual_decision(
    service: &ApplicationService,
    store: &PersistenceStore,
    task_id: Uuid,
    research_run_id: Uuid,
) -> ApplicationResult<()> {
''',
    "manual conflict reconciliation service access",
)
replace_once(
    "crates/application/src/p4_workbench.rs",
    "match finalize_successful_research(store, &recovered).await {",
    "match service.finalize_p4_research_task(&recovered).await {",
    "manual conflict ResearchService finalization",
)

verifier_path = ROOT / "scripts/verify-research-service.mjs"
verifier = verifier_path.read_text(encoding="utf-8")
required_anchor = '  "crates/application/src/use_cases/research/openai_gateway/validation.rs",\n'
required_add = required_anchor + ''.join(
    f'  "crates/application/src/use_cases/research/p4_worker/{name}",\n'
    for name in ["mod.rs", "context.rs", "execution.rs", "transitions.rs"]
)
if verifier.count(required_anchor) != 1:
    raise RuntimeError("AT4 verifier required files anchor mismatch")
verifier = verifier.replace(required_anchor, required_add, 1)
verifier = verifier.replace(
    'const pipeline = read("crates/application/src/use_cases/research/fact_pipeline/mod.rs");\n',
    'const pipeline = read("crates/application/src/use_cases/research/fact_pipeline/mod.rs");\n'
    'const p4Worker = read("crates/application/src/use_cases/research/p4_worker/execution.rs");\n'
    'const p4Transitions = read("crates/application/src/use_cases/research/p4_worker/transitions.rs");\n'
    'const p4Orchestration = read("crates/application/src/p4_orchestration.rs");\n'
    'const p4Workbench = read("crates/application/src/p4_workbench.rs");\n',
    1,
)
legacy_block = '''for (const path of ["crates/application/src/p4_orchestration.rs", "crates/application/src/p4_workbench.rs"]) {
  check(existsSync(join(root, path)), `后续 R3-07 职责被提前删除：${path}`);
}
'''
new_block = '''check(existsSync(join(root, "crates/application/src/p4_orchestration.rs")), "跨 Prediction/Research 的 P4 dispatcher 被提前删除");
check(existsSync(join(root, "crates/application/src/p4_workbench.rs")), "人工冲突裁决被提前删除");
check(service.includes("fn execute_p4_research_task"), "ResearchService 缺少 P4 Research worker 执行职责");
check(service.includes("fn finalize_successful_research"), "ResearchService 缺少 Research 成功收口职责");
check(facade.includes("fn execute_p4_research_task"), "Application Research facade 缺少 P4 Research worker 委托");
check(facade.includes("fn finalize_p4_research_task"), "Application Research facade 缺少人工裁决复用的 Research 收口入口");
check(p4Worker.includes("openai_gateway::execute_p4_openai_research"), "P4 Research worker 未复用已迁移 OpenAI Gateway");
check(p4Worker.includes("PredictionWorkflowPort"), "P4 Research worker 未通过 PredictionWorkflowPort 管理冻结任务状态");
check(p4Worker.includes("ResearchArtifactPort"), "P4 Research worker 未通过 ResearchArtifactPort 管理 research run");
check(p4Transitions.includes("JobQueuePort"), "P4 Research worker 未通过 JobQueuePort 安排 freeze job");
check(p4Orchestration.includes("self.execute_p4_research_task(payload.task_id, job_id)"), "P4 dispatcher 未委托 ResearchService 执行 Research job");
for (const token of ["OpenAiResearchCommand", "ResearchRunDraft", "ResearchRunStatus", "research_dynamic_context", "fn finalize_successful_research", "fn block_partial_research", "fn transition_missed", "execute_p4_openai_research("]) {
  check(!p4Orchestration.includes(token), `旧 p4_orchestration.rs 仍持有 Research worker 业务逻辑：${token}`);
}
check(!p4Workbench.includes("p4_orchestration::finalize_successful_research"), "人工冲突裁决仍依赖旧 P4 orchestration Research helper");
check(p4Workbench.includes("service.finalize_p4_research_task(&recovered).await"), "人工冲突裁决未复用 ResearchService 成功收口职责");
'''
if verifier.count(legacy_block) != 1:
    raise RuntimeError("AT4 verifier legacy boundary block mismatch")
verifier = verifier.replace(legacy_block, new_block, 1)
verifier = verifier.replace(
    'console.log(`Research Service AT3 验证通过：${researchFiles.length} 个 Service/Use Case Rust 文件，9 个公开 Research API 已进入 ResearchService/Ports；Fact Pipeline 保持 ${pipelineFiles.length} 个模块，OpenAI Gateway execution 已拆入 ResearchService，旧 p4_persistence.rs / fact_pipeline.rs / openai_research.rs 均已删除。`);',
    'console.log(`Research Service AT4 验证通过：${researchFiles.length} 个 Service/Use Case Rust 文件；9 个公开 Research API 保持兼容，Fact Pipeline、OpenAI Gateway 与 P4 Research worker 均进入 ResearchService/Ports，根 p4_orchestration.rs 仅保留跨服务 dispatcher/worker loop，人工冲突裁决继续留给后续 Atomic Task。`);',
    1,
)
verifier_path.write_text(verifier, encoding="utf-8", newline="\n")

at4_record = "- R3-07 Atomic Task 4 已迁移 P4 Research worker：Research task 状态机、联网动态上下文、research run 恢复/执行、Partial/Blocked/Missed/ReadyToFreeze 收口与 freeze job 安排按职责拆入 `use_cases/research/p4_worker/`，并由 ResearchService 经既有 `PredictionWorkflowPort`、`ResearchArtifactPort`、`ResearchGatewayAuditPort`、Fact Pipeline 与 `JobQueuePort` 协作；根 `p4_orchestration.rs` 仅保留跨 Research/Prediction 的 job claim/complete/fail、dispatcher 与 worker loop，Prediction freeze 执行仍委托 PredictionService。`p4_workbench.rs` 的人工冲突裁决逻辑未迁移，仅将成功后的 Research 收口改为复用 ResearchService。公共 API、job type、状态语义、SQL、Schema、迁移、生产依赖与模型边界未改变。AT4 仍处于 `IN_PROGRESS`，须通过 staging 全量硬门禁及正式 Public Platform CI 后才能关闭。"
insert_after("README.md", "R3-07 Atomic Task 3 已正式关闭为 `DONE`", at4_record)

r3_record = "- Atomic Task 4 已迁移 P4 Research worker：Research 状态机、动态上下文、OpenAI Gateway 调用、run 恢复与 freeze handoff 拆入 `use_cases/research/p4_worker/`；ResearchService 只经既有 Ports 协作。根 `p4_orchestration.rs` 收敛为跨服务 dispatcher/worker loop，Prediction freeze 仍由 PredictionService 执行；`p4_workbench.rs` 人工冲突裁决保持后续边界，只复用 ResearchService 的成功收口。AT4 当前 `IN_PROGRESS`，正式 Public Platform CI 通过前不得标记 `DONE`。"
insert_after("docs/modular-rewrite/R03-application-services/README.md", "Atomic Task 3 已正式关闭为 `DONE`", r3_record)

print("R3-07 Atomic Task 4 P4 Research worker migration generated")
