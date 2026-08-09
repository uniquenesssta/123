from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def read(path: str) -> str:
    return (ROOT / path).read_text(encoding="utf-8")


def write(path: str, content: str) -> None:
    target = ROOT / path
    target.parent.mkdir(parents=True, exist_ok=True)
    target.write_text(content, encoding="utf-8", newline="\n")


def replace_once(path: str, old: str, new: str, label: str) -> None:
    target = ROOT / path
    text = target.read_text(encoding="utf-8")
    count = text.count(old)
    if count != 1:
        raise RuntimeError(f"AT5 anchor mismatch for {label}: expected 1, found {count}")
    target.write_text(text.replace(old, new, 1), encoding="utf-8", newline="\n")


write(
    "crates/application/src/use_cases/research/p4_manual_conflict/mod.rs",
    '''use crate::ports::{
    analytics::JobQueuePort,
    prediction::PredictionWorkflowPort,
    research::{ResearchArtifactPort, ResearchManualConflictPort},
};
use crate::ApplicationResult;
use chrono::Utc;
use football_domain::{P4TaskWorkspace, ResolveP4ConflictCommand};

mod decision;
mod reconciliation;

pub(crate) struct P4ManualConflictAccess<'a> {
    pub workflow: &'a dyn PredictionWorkflowPort,
    pub jobs: &'a dyn JobQueuePort,
    pub artifacts: &'a dyn ResearchArtifactPort,
    pub manual: &'a dyn ResearchManualConflictPort,
}

pub(crate) async fn resolve(
    access: P4ManualConflictAccess<'_>,
    command: ResolveP4ConflictCommand,
) -> ApplicationResult<P4TaskWorkspace> {
    let workspace = access.workflow.read_task_workspace(command.task_id).await?;
    let prepared = decision::prepare(&workspace, command, Utc::now())?;
    let (task_id, research_run_id) = match prepared {
        decision::PreparedDecision::Existing {
            task_id,
            research_run_id,
        } => (task_id, research_run_id),
        decision::PreparedDecision::Append {
            task_id,
            research_run_id,
            draft,
        } => {
            access.manual.append_manual_route_override(&draft).await?;
            (task_id, research_run_id)
        }
    };

    reconciliation::reconcile(&access, task_id, research_run_id).await?;
    Ok(access.workflow.read_task_workspace(task_id).await?)
}
''',
)

write(
    "crates/application/src/use_cases/research/p4_manual_conflict/decision.rs",
    '''use crate::{ApplicationError, ApplicationResult};
use chrono::{DateTime, Utc};
use football_domain::{
    P4FreezeTaskState, P4ManualConflictDecisionKind, P4ManualRouteOverrideDraft, P4TaskWorkspace,
    ResolveP4ConflictCommand,
};
use serde_json::Value;
use std::collections::BTreeSet;
use uuid::Uuid;

pub(super) enum PreparedDecision {
    Existing {
        task_id: Uuid,
        research_run_id: Uuid,
    },
    Append {
        task_id: Uuid,
        research_run_id: Uuid,
        draft: P4ManualRouteOverrideDraft,
    },
}

pub(super) fn prepare(
    workspace: &P4TaskWorkspace,
    command: ResolveP4ConflictCommand,
    now: DateTime<Utc>,
) -> ApplicationResult<PreparedDecision> {
    let task = &workspace.task;
    let research_run_id = task
        .research_run_id
        .ok_or_else(|| ApplicationError::Validation("当前任务缺少研究任务记录".to_string()))?;
    let conflict = workspace
        .conflicts
        .iter()
        .find(|item| item.id == command.conflict_id)
        .ok_or_else(|| ApplicationError::Validation("冲突不属于当前任务".to_string()))?;
    let mut requested_evidence_ids = command.selected_evidence_ids.clone();
    requested_evidence_ids.sort_unstable();
    requested_evidence_ids.dedup();
    let note = command
        .note
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string);

    if let Some(existing_kind) = conflict.manual_decision_kind.as_deref() {
        let mut existing_evidence_ids = conflict.selected_evidence_ids.clone();
        existing_evidence_ids.sort_unstable();
        existing_evidence_ids.dedup();
        if existing_kind == command.decision_kind.as_str()
            && existing_evidence_ids == requested_evidence_ids
            && conflict.manual_decision_note == note
        {
            return Ok(PreparedDecision::Existing {
                task_id: task.id,
                research_run_id,
            });
        }
        return Err(ApplicationError::Validation(
            "该冲突已经存在人工决策；不可覆盖历史决策".to_string(),
        ));
    }

    if !matches!(
        task.state,
        P4FreezeTaskState::ResearchPartial | P4FreezeTaskState::Blocked
    ) {
        return Err(ApplicationError::Validation(format!(
            "只有RESEARCH_PARTIAL或BLOCKED任务可以人工处理冲突，当前状态为{}",
            task.state.as_str()
        )));
    }
    if now >= task.data_cutoff_at {
        return Err(ApplicationError::Validation(
            "数据截止时间已经到达，不能在截止后改变正式证据选择".to_string(),
        ));
    }
    if conflict.evaluation_status.as_deref() != Some("manual_required") {
        return Err(ApplicationError::Validation(
            "只有等待人工确认的冲突可以由用户处理".to_string(),
        ));
    }

    let conflict_evidence = conflict
        .evidence_ids
        .iter()
        .copied()
        .collect::<BTreeSet<_>>();
    let route = workspace
        .routes
        .iter()
        .find(|route| {
            route.route_status == "blocked_conflict"
                && route.field_key == conflict.field_key
                && route
                    .selected_evidence_ids
                    .iter()
                    .any(|evidence_id| conflict_evidence.contains(evidence_id))
        })
        .ok_or_else(|| ApplicationError::Validation("未找到与冲突对应的阻断路由".to_string()))?;

    let (selected_evidence_ids, selected_value, verification_state, route_status, reason) =
        match &command.decision_kind {
            P4ManualConflictDecisionKind::SelectEvidence => {
                let selected = requested_evidence_ids.clone();
                if selected.is_empty() {
                    return Err(ApplicationError::Validation(
                        "请选择至少一条证据".to_string(),
                    ));
                }
                if selected
                    .iter()
                    .any(|evidence_id| !conflict_evidence.contains(evidence_id))
                {
                    return Err(ApplicationError::Validation(
                        "所选证据不属于当前冲突组".to_string(),
                    ));
                }
                let values = selected
                    .iter()
                    .map(|evidence_id| {
                        workspace
                            .evidence
                            .iter()
                            .find(|item| item.id == *evidence_id)
                            .map(|item| item.value.clone())
                            .ok_or_else(|| {
                                ApplicationError::Validation(
                                    "所选证据在研究账本中不存在".to_string(),
                                )
                            })
                    })
                    .collect::<ApplicationResult<Vec<_>>>()?;
                let first = values.first().cloned().ok_or_else(|| {
                    ApplicationError::Validation("所选证据缺少事实值".to_string())
                })?;
                if values.iter().any(|value| value != &first) {
                    return Err(ApplicationError::Validation(
                        "一次只能选择事实值完全一致的一组证据".to_string(),
                    ));
                }
                (
                    selected,
                    first,
                    "PROBABLE".to_string(),
                    "routed".to_string(),
                    "用户在截止前从保留的冲突证据中选择一致事实；人工决策固定降级为PROBABLE"
                        .to_string(),
                )
            }
            P4ManualConflictDecisionKind::AcceptUnknown => {
                if !requested_evidence_ids.is_empty() {
                    return Err(ApplicationError::Validation(
                        "接受未知时不能同时选择证据".to_string(),
                    ));
                }
                (
                    Vec::new(),
                    Value::Null,
                    "NOT_FOUND".to_string(),
                    "missing".to_string(),
                    "用户在截止前确认当前冲突无法安全裁决，按NOT_FOUND进入冻结账本".to_string(),
                )
            }
        };

    let decision_key = format!(
        "p4-manual-conflict:{}:{}:{}",
        task.id,
        conflict.id,
        command.decision_kind.as_str()
    );
    Ok(PreparedDecision::Append {
        task_id: task.id,
        research_run_id,
        draft: P4ManualRouteOverrideDraft {
            task_id: task.id,
            research_run_id,
            conflict_id: conflict.id,
            route_key: route.route_key.clone(),
            field_key: route.field_key.clone(),
            target_module: route.target_module.clone(),
            target_slot: route.target_slot.clone(),
            entity_type: Some(conflict.entity_type.clone()),
            entity_id: conflict.entity_id,
            decision_kind: command.decision_kind,
            selected_evidence_ids,
            selected_value,
            verification_state,
            route_status,
            reason,
            actor: "local_user".to_string(),
            note,
            idempotency_key: decision_key,
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn manual_resolution_decision_names_are_stable() {
        assert_eq!(
            serde_json::to_string(&P4ManualConflictDecisionKind::SelectEvidence).unwrap(),
            "\"select_evidence\""
        );
        assert_eq!(
            serde_json::to_string(&P4ManualConflictDecisionKind::AcceptUnknown).unwrap(),
            "\"accept_unknown\""
        );
    }
}
''',
)

write(
    "crates/application/src/use_cases/research/p4_manual_conflict/reconciliation.rs",
    '''use super::P4ManualConflictAccess;
use crate::use_cases::research::p4_worker;
use crate::{ApplicationError, ApplicationResult};
use football_domain::{
    P4FreezeTaskState, P4FreezeTaskTransition, ResearchRunEventDraft, ResearchRunStatus,
};
use serde_json::json;
use uuid::Uuid;

pub(super) async fn reconcile(
    access: &P4ManualConflictAccess<'_>,
    task_id: Uuid,
    research_run_id: Uuid,
) -> ApplicationResult<()> {
    let route_readiness = access.manual.route_readiness(task_id).await?;
    if route_readiness.ready {
        access
            .artifacts
            .record_run_event(&ResearchRunEventDraft {
                research_run_id,
                idempotency_key: format!("manual-review-succeeded:{task_id}"),
                status: ResearchRunStatus::Succeeded,
                response_id: None,
                model_id: None,
                token_usage: json!({}),
                error_category: None,
                error_message: None,
                payload: json!({
                    "stage": "G",
                    "task_id": task_id,
                    "reason": "all immutable routes passed after append-only manual conflict decisions"
                }),
            })
            .await?;

        for _ in 0..4 {
            let current = access.workflow.read_freeze_task(task_id).await?;
            let recovered = match current.state {
                P4FreezeTaskState::ResearchPartial | P4FreezeTaskState::Blocked => {
                    match access
                        .workflow
                        .transition_freeze_task(
                            task_id,
                            &P4FreezeTaskTransition {
                                task_id,
                                expected_state: current.state,
                                next_state: P4FreezeTaskState::ResearchSucceeded,
                                reason: "人工冲突处理完成，全部事实路由重新通过门禁".to_string(),
                                blockers: json!([]),
                                payload: serde_json::to_value(&route_readiness)?,
                                research_run_id: None,
                                research_job_id: None,
                                freeze_job_id: None,
                                snapshot_id: None,
                            },
                        )
                        .await
                    {
                        Ok(task) => task,
                        Err(error) => {
                            let latest = access.workflow.read_freeze_task(task_id).await?;
                            if latest.state != current.state {
                                continue;
                            }
                            return Err(error.into());
                        }
                    }
                }
                P4FreezeTaskState::ResearchSucceeded => current,
                P4FreezeTaskState::ReadyToFreeze
                | P4FreezeTaskState::Freezing
                | P4FreezeTaskState::Frozen
                | P4FreezeTaskState::Missed
                | P4FreezeTaskState::Failed
                | P4FreezeTaskState::Cancelled => return Ok(()),
                other => {
                    return Err(ApplicationError::Validation(format!(
                        "人工冲突决策完成后无法从状态{}恢复冻结链",
                        other.as_str()
                    )))
                }
            };

            match p4_worker::finalize_successful_research(
                access.workflow,
                access.jobs,
                &recovered,
            )
            .await
            {
                Ok(_) => return Ok(()),
                Err(error) => {
                    let latest = access.workflow.read_freeze_task(task_id).await?;
                    if matches!(
                        latest.state,
                        P4FreezeTaskState::ReadyToFreeze
                            | P4FreezeTaskState::Freezing
                            | P4FreezeTaskState::Frozen
                    ) {
                        return Ok(());
                    }
                    if latest.state == P4FreezeTaskState::ResearchSucceeded {
                        continue;
                    }
                    return Err(error);
                }
            }
        }
        return Err(ApplicationError::Validation(
            "人工冲突处理后的任务状态持续发生并发变化，请刷新工作台确认最终状态".to_string(),
        ));
    }

    let current = access.workflow.read_freeze_task(task_id).await?;
    match current.state {
        P4FreezeTaskState::ResearchPartial => {
            match access
                .workflow
                .transition_freeze_task(
                    task_id,
                    &P4FreezeTaskTransition {
                        task_id,
                        expected_state: P4FreezeTaskState::ResearchPartial,
                        next_state: P4FreezeTaskState::Blocked,
                        reason: "人工处理后仍存在其他阻断项".to_string(),
                        blockers: serde_json::to_value(&route_readiness.blockers)?,
                        payload: serde_json::to_value(&route_readiness)?,
                        research_run_id: None,
                        research_job_id: None,
                        freeze_job_id: None,
                        snapshot_id: None,
                    },
                )
                .await
            {
                Ok(_) => Ok(()),
                Err(error) => {
                    let latest = access.workflow.read_freeze_task(task_id).await?;
                    if latest.state != P4FreezeTaskState::ResearchPartial {
                        Ok(())
                    } else {
                        Err(error.into())
                    }
                }
            }
        }
        P4FreezeTaskState::Blocked
        | P4FreezeTaskState::ResearchSucceeded
        | P4FreezeTaskState::ReadyToFreeze
        | P4FreezeTaskState::Freezing
        | P4FreezeTaskState::Frozen
        | P4FreezeTaskState::Missed
        | P4FreezeTaskState::Failed
        | P4FreezeTaskState::Cancelled => Ok(()),
        other => Err(ApplicationError::Validation(format!(
            "人工处理后无法从状态{}登记剩余阻断项",
            other.as_str()
        ))),
    }
}
''',
)

replace_once(
    "crates/application/src/use_cases/research/mod.rs",
    "pub(crate) mod p4_worker;\n",
    "pub(crate) mod p4_manual_conflict;\npub(crate) mod p4_worker;\n",
    "manual conflict module registration",
)

replace_once(
    "crates/application/src/ports/research/mod.rs",
    '''    EvidenceRouteDraft, EvidenceRouteRecord, FactPipelineContext, OpenAiAttemptDraft,
    OpenAiAttemptRecord, OpenAiUsageTotals, PromptVersionDraft, PromptVersionRecord,
''',
    '''    EvidenceRouteDraft, EvidenceRouteRecord, FactPipelineContext, OpenAiAttemptDraft,
    OpenAiAttemptRecord, OpenAiUsageTotals, P4FreezeReadiness, P4ManualRouteOverrideDraft,
    P4ManualRouteOverrideRecord, PromptVersionDraft, PromptVersionRecord,
''',
    "manual conflict domain imports",
)

replace_once(
    "crates/application/src/ports/research/mod.rs",
    '''#[async_trait]
pub trait FactPipelinePort: Send + Sync {
''',
    '''#[async_trait]
pub trait ResearchManualConflictPort: Send + Sync {
    async fn append_manual_route_override(
        &self,
        draft: &P4ManualRouteOverrideDraft,
    ) -> PortResult<P4ManualRouteOverrideRecord>;
    async fn route_readiness(&self, task_id: Uuid) -> PortResult<P4FreezeReadiness>;
}

#[async_trait]
pub trait FactPipelinePort: Send + Sync {
''',
    "manual conflict port",
)

replace_once(
    "crates/application/src/composition/adapters/research.rs",
    '''        FactPipelinePort, ResearchArtifactPort, ResearchEvidenceLedgerPort,
        ResearchGatewayAuditPort, SerializedConflictEventPayload,
''',
    '''        FactPipelinePort, ResearchArtifactPort, ResearchEvidenceLedgerPort,
        ResearchGatewayAuditPort, ResearchManualConflictPort, SerializedConflictEventPayload,
''',
    "manual conflict adapter port import",
)

replace_once(
    "crates/application/src/composition/adapters/research.rs",
    '''    EvidenceRouteDraft, EvidenceRouteRecord, FactPipelineContext, OpenAiAttemptDraft,
    OpenAiAttemptRecord, OpenAiUsageTotals, PromptVersionDraft, PromptVersionRecord,
''',
    '''    EvidenceRouteDraft, EvidenceRouteRecord, FactPipelineContext, OpenAiAttemptDraft,
    OpenAiAttemptRecord, OpenAiUsageTotals, P4FreezeReadiness, P4ManualRouteOverrideDraft,
    P4ManualRouteOverrideRecord, PromptVersionDraft, PromptVersionRecord,
''',
    "manual conflict adapter domain imports",
)

adapter_path = ROOT / "crates/application/src/composition/adapters/research.rs"
adapter = adapter_path.read_text(encoding="utf-8")
adapter_insert = '''

#[async_trait]
impl ResearchManualConflictPort for ActiveDatabase {
    async fn append_manual_route_override(
        &self,
        draft: &P4ManualRouteOverrideDraft,
    ) -> PortResult<P4ManualRouteOverrideRecord> {
        self.transition_store()
            .append_p4_manual_route_override(draft)
            .await
            .map_err(map_persistence_error)
    }

    async fn route_readiness(&self, task_id: Uuid) -> PortResult<P4FreezeReadiness> {
        self.transition_store()
            .p4_route_readiness(task_id)
            .await
            .map_err(map_persistence_error)
    }
}
'''
anchor = "\n#[async_trait]\nimpl FactPipelinePort for ActiveDatabase {"
if adapter.count(anchor) != 1:
    raise RuntimeError("AT5 adapter insertion anchor mismatch")
adapter_path.write_text(adapter.replace(anchor, adapter_insert + anchor, 1), encoding="utf-8", newline="\n")

replace_once(
    "crates/application/src/services/research/service.rs",
    '''use crate::ports::{
    analytics::JobQueuePort,
    prediction::PredictionWorkflowPort,
    research::{ResearchArtifactPort, ResearchEvidenceLedgerPort, ResearchGatewayAuditPort},
};
''',
    '''use crate::ports::research::{
    ResearchArtifactPort, ResearchEvidenceLedgerPort, ResearchGatewayAuditPort,
};
''',
    "research service obsolete finalizer ports",
)

replace_once(
    "crates/application/src/services/research/service.rs",
    '''    openai_gateway::{self, OpenAiResearchCommand},
    p4_worker::{self, P4ResearchWorkerAccess},
''',
    '''    openai_gateway::{self, OpenAiResearchCommand},
    p4_manual_conflict::{self, P4ManualConflictAccess},
    p4_worker::{self, P4ResearchWorkerAccess},
''',
    "research service manual use case import",
)

replace_once(
    "crates/application/src/services/research/service.rs",
    '''    P4FreezeTaskRecord, PromptVersionDraft, PromptVersionRecord, ResearchRunDraft,
    ResearchRunEventDraft, ResearchRunRecord, SchemaVersionDraft, SchemaVersionRecord,
''',
    '''    P4TaskWorkspace, PromptVersionDraft, PromptVersionRecord, ResearchRunDraft,
    ResearchRunEventDraft, ResearchRunRecord, ResolveP4ConflictCommand, SchemaVersionDraft,
    SchemaVersionRecord,
''',
    "research service manual domain types",
)

service_path = ROOT / "crates/application/src/services/research/service.rs"
service = service_path.read_text(encoding="utf-8")
old_finalizer = '''

    pub(crate) async fn finalize_successful_research(
        &self,
        workflow: &dyn PredictionWorkflowPort,
        jobs: &dyn JobQueuePort,
        task: &P4FreezeTaskRecord,
    ) -> ApplicationResult<P4FreezeTaskRecord> {
        p4_worker::finalize_successful_research(workflow, jobs, task).await
    }
'''
if service.count(old_finalizer) != 1:
    raise RuntimeError("AT5 obsolete ResearchService finalizer anchor mismatch")
manual_service = '''

    pub(crate) async fn resolve_p4_conflict(
        &self,
        access: P4ManualConflictAccess<'_>,
        command: ResolveP4ConflictCommand,
    ) -> ApplicationResult<P4TaskWorkspace> {
        p4_manual_conflict::resolve(access, command).await
    }
'''
service_path.write_text(service.replace(old_finalizer, manual_service, 1), encoding="utf-8", newline="\n")

replace_once(
    "crates/application/src/services/research/facade.rs",
    '''use crate::composition::ActiveDatabase;
use crate::use_cases::research::p4_worker::P4ResearchWorkerAccess;
''',
    '''use crate::composition::ActiveDatabase;
use crate::use_cases::research::{
    p4_manual_conflict::P4ManualConflictAccess,
    p4_worker::P4ResearchWorkerAccess,
};
''',
    "research facade manual access import",
)

replace_once(
    "crates/application/src/services/research/facade.rs",
    '''    P4FreezeTaskRecord, PromptVersionDraft, PromptVersionRecord, ResearchRunDraft,
    ResearchRunEventDraft, ResearchRunRecord, SchemaVersionDraft, SchemaVersionRecord,
''',
    '''    P4TaskWorkspace, PromptVersionDraft, PromptVersionRecord, ResearchRunDraft,
    ResearchRunEventDraft, ResearchRunRecord, ResolveP4ConflictCommand, SchemaVersionDraft,
    SchemaVersionRecord,
''',
    "research facade manual domain types",
)

facade_path = ROOT / "crates/application/src/services/research/facade.rs"
facade = facade_path.read_text(encoding="utf-8")
old_facade_finalizer = '''

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
if facade.count(old_facade_finalizer) != 1:
    raise RuntimeError("AT5 obsolete facade finalizer anchor mismatch")
manual_facade = '''

    pub async fn resolve_p4_conflict(
        &self,
        command: ResolveP4ConflictCommand,
    ) -> ApplicationResult<P4TaskWorkspace> {
        let session = self.research_session().await?;
        let access = P4ManualConflictAccess {
            workflow: &session,
            jobs: &session,
            artifacts: &session,
            manual: &session,
        };
        self.research.resolve_p4_conflict(access, command).await
    }
'''
facade_path.write_text(facade.replace(old_facade_finalizer, manual_facade, 1), encoding="utf-8", newline="\n")

replace_once(
    "crates/application/src/lib.rs",
    "mod p4_orchestration;\nmod p4_workbench;\n",
    "mod p4_orchestration;\n",
    "application p4 workbench module removal",
)

workbench_path = ROOT / "crates/application/src/p4_workbench.rs"
if not workbench_path.exists():
    raise RuntimeError("AT5 expected p4_workbench.rs to exist")
workbench_path.unlink()

replace_once(
    "scripts/verify-research-service.mjs",
    '''  "execute_p4_openai_research",
];
''',
    '''  "execute_p4_openai_research",
  "resolve_p4_conflict",
];
''',
    "research verifier public manual API",
)

replace_once(
    "scripts/verify-research-service.mjs",
    '''  "crates/application/src/use_cases/research/p4_worker/transitions.rs",
  "crates/application/src/composition/adapters/research.rs",
''',
    '''  "crates/application/src/use_cases/research/p4_worker/transitions.rs",
  "crates/application/src/use_cases/research/p4_manual_conflict/mod.rs",
  "crates/application/src/use_cases/research/p4_manual_conflict/decision.rs",
  "crates/application/src/use_cases/research/p4_manual_conflict/reconciliation.rs",
  "crates/application/src/composition/adapters/research.rs",
''',
    "research verifier manual files",
)

replace_once(
    "scripts/verify-research-service.mjs",
    '''check(!existsSync(join(root, "crates/application/src/openai_research.rs")), "旧 openai_research.rs 仍残留 OpenAI Research owner 或空转发层");
''',
    '''check(!existsSync(join(root, "crates/application/src/openai_research.rs")), "旧 openai_research.rs 仍残留 OpenAI Research owner 或空转发层");
check(!existsSync(join(root, "crates/application/src/p4_workbench.rs")), "旧 p4_workbench.rs 仍残留人工冲突裁决 owner 或空转发层");
''',
    "research verifier old workbench absence",
)

replace_once(
    "scripts/verify-research-service.mjs",
    '''const p4Transitions = read("crates/application/src/use_cases/research/p4_worker/transitions.rs");
const p4Orchestration = read("crates/application/src/p4_orchestration.rs");
const p4Workbench = read("crates/application/src/p4_workbench.rs");
''',
    '''const p4Transitions = read("crates/application/src/use_cases/research/p4_worker/transitions.rs");
const p4Manual = read("crates/application/src/use_cases/research/p4_manual_conflict/mod.rs");
const p4Decision = read("crates/application/src/use_cases/research/p4_manual_conflict/decision.rs");
const p4Reconciliation = read("crates/application/src/use_cases/research/p4_manual_conflict/reconciliation.rs");
const p4Orchestration = read("crates/application/src/p4_orchestration.rs");
''',
    "research verifier manual sources",
)

replace_once(
    "scripts/verify-research-service.mjs",
    '''check(ports.includes("trait ResearchGatewayAuditPort"), "Research Ports 缺少 ResearchGatewayAuditPort");
''',
    '''check(ports.includes("trait ResearchGatewayAuditPort"), "Research Ports 缺少 ResearchGatewayAuditPort");
check(ports.includes("trait ResearchManualConflictPort"), "Research Ports 缺少 ResearchManualConflictPort");
for (const capability of ["append_manual_route_override", "route_readiness"]) {
  check(ports.includes(`fn ${capability}`), `ResearchManualConflictPort 缺少能力：${capability}`);
}
''',
    "research verifier manual port checks",
)

replace_once(
    "scripts/verify-research-service.mjs",
    '''check(adapter.includes("impl ResearchGatewayAuditPort for ActiveDatabase"), "Research adapter 缺少 Gateway Audit 实现");
''',
    '''check(adapter.includes("impl ResearchGatewayAuditPort for ActiveDatabase"), "Research adapter 缺少 Gateway Audit 实现");
check(adapter.includes("impl ResearchManualConflictPort for ActiveDatabase"), "Research adapter 缺少 Manual Conflict 实现");
for (const token of [".append_p4_manual_route_override(draft)", ".p4_route_readiness(task_id)"]) {
  check(adapter.includes(token), `Manual Conflict adapter 未复用既有持久化能力：${token}`);
}
''',
    "research verifier manual adapter checks",
)

replace_once(
    "scripts/verify-research-service.mjs",
    '''check(existsSync(join(root, "crates/application/src/p4_workbench.rs")), "人工冲突裁决被提前删除");
''',
    '''check(!lib.includes("mod p4_workbench;"), "Application 根模块仍登记旧 p4_workbench owner");
''',
    "research verifier workbench owner migrated",
)

replace_once(
    "scripts/verify-research-service.mjs",
    '''check(facade.includes("fn finalize_p4_research_task"), "Application Research facade 缺少人工裁决复用的 Research 收口入口");
''',
    '''check(service.includes("fn resolve_p4_conflict"), "ResearchService 缺少人工冲突裁决职责");
check(facade.includes("pub async fn resolve_p4_conflict"), "Application Research facade 缺少公共人工冲突裁决入口");
check(facade.includes("ApplicationResult<P4TaskWorkspace>"), "resolve_p4_conflict 返回契约发生变化");
check(!service.includes("fn finalize_successful_research"), "AT5 后仍残留仅供旧 workbench 的 ResearchService finalizer 转发");
check(!facade.includes("fn finalize_p4_research_task"), "AT5 后仍残留仅供旧 workbench 的 facade finalizer 转发");
''',
    "research verifier manual public contract",
)

replace_once(
    "scripts/verify-research-service.mjs",
    '''check(!p4Workbench.includes("p4_orchestration::finalize_successful_research"), "人工冲突裁决仍依赖旧 P4 orchestration Research helper");
check(p4Workbench.includes("service.finalize_p4_research_task(&recovered).await"), "人工冲突裁决未复用 ResearchService 成功收口职责");
''',
    '''check(p4Manual.includes("P4ManualConflictAccess"), "人工冲突裁决缺少 Ports 组合访问边界");
for (const token of ["PredictionWorkflowPort", "JobQueuePort", "ResearchArtifactPort", "ResearchManualConflictPort"]) {
  check(p4Manual.includes(token), `人工冲突裁决访问边界缺少：${token}`);
}
check(p4Decision.includes("Utc"), "人工冲突裁决未保留截止时间判断");
check(p4Decision.includes("p4-manual-conflict:{}:{}:{}"), "人工冲突裁决幂等键格式发生变化");
check(p4Decision.includes('actor: "local_user"'), "人工冲突裁决 actor 语义发生变化");
check(p4Decision.includes('"PROBABLE"') && p4Decision.includes('"NOT_FOUND"'), "人工冲突裁决 verification 语义发生变化");
check(p4Reconciliation.includes("for _ in 0..4"), "人工冲突裁决并发恢复重试次数发生变化");
check(p4Reconciliation.includes("manual-review-succeeded:{task_id}"), "人工裁决 Research run event 幂等键发生变化");
check(p4Reconciliation.includes("p4_worker::finalize_successful_research"), "人工冲突裁决未复用 AT4 Research 成功收口");
''',
    "research verifier manual migration checks",
)

replace_once(
    "scripts/verify-research-service.mjs",
    '''console.log(`Research Service AT4 验证通过：${researchFiles.length} 个 Service/Use Case Rust 文件；9 个公开 Research API 保持兼容，Fact Pipeline、OpenAI Gateway 与 P4 Research worker 均进入 ResearchService/Ports，根 p4_orchestration.rs 仅保留跨服务 dispatcher/worker loop，人工冲突裁决继续留给后续 Atomic Task。`);
''',
    '''console.log(`Research Service AT5 验证通过：${researchFiles.length} 个 Service/Use Case Rust 文件；10 个公开 Research API 保持兼容，Fact Pipeline、OpenAI Gateway、P4 Research worker 与人工冲突裁决均进入 ResearchService/Ports，根 p4_orchestration.rs 仅保留跨服务 dispatcher/worker loop，旧 p4_workbench.rs 已删除。`);
''',
    "research verifier AT5 summary",
)

replace_once(
    "README.md",
    "AT4 仍处于 `IN_PROGRESS`，须通过 staging 全量硬门禁及正式 Public Platform CI 后才能关闭。",
    "AT4 已正式关闭为 `DONE`：clean implementation `89a3c68ad50f7766d6db5d214c0fa5a39c1a6c72` 的同源码树验收提交 `75e76dea9a9ab7980f64374b4f27410ee221f8f9` 已通过 Public Platform CI run `31316144230` / Windows Automated job `93251603683`，validation evidence upload 成功；artifact `9039129007` 大小 `14264776` 字节，SHA-256 `fe3ef81501cb2c7d57302f8e03e9b0753f82138d8dc54356e39fc5d0ab31f68f`。Atomic Task 5 已进入 `IN_PROGRESS`：人工冲突裁决迁入 ResearchService / `use_cases/research/p4_manual_conflict/`，只经既有 Prediction/Job/Research Ports 协作；SQL、Schema、迁移、生产依赖、公共 `resolve_p4_conflict` 契约、截止时间、幂等、append-only 与状态机语义保持不变。AT5_HARD_GATE_PENDING",
    "root README AT4 closeout and AT5 start",
)

replace_once(
    "docs/modular-rewrite/R03-application-services/README.md",
    "AT4 当前 `IN_PROGRESS`，正式 Public Platform CI 通过前不得标记 `DONE`。",
    "AT4 已正式关闭为 `DONE`：clean implementation `89a3c68ad50f7766d6db5d214c0fa5a39c1a6c72` 的同源码树验收提交 `75e76dea9a9ab7980f64374b4f27410ee221f8f9` 已通过 Public Platform CI run `31316144230` / Windows Automated job `93251603683`，validation evidence upload 成功；artifact `9039129007` 大小 `14264776` 字节，SHA-256 `fe3ef81501cb2c7d57302f8e03e9b0753f82138d8dc54356e39fc5d0ab31f68f`。Atomic Task 5 已进入 `IN_PROGRESS`：人工冲突裁决迁入 ResearchService / `use_cases/research/p4_manual_conflict/`，公共 `resolve_p4_conflict` 与全部既有人工裁决语义保持不变。AT5_HARD_GATE_PENDING",
    "R3 README AT4 closeout and AT5 start",
)

print("R3-07 Atomic Task 5 generated")
