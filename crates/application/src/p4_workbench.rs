use super::{
    p4_orchestration::finalize_successful_research, ApplicationError, ApplicationResult,
    ApplicationService,
};
use crate::PersistenceStore;
use chrono::Utc;
use football_domain::{
    P4FreezeTaskState, P4FreezeTaskTransition, P4ManualConflictDecisionKind,
    P4ManualRouteOverrideDraft, P4MatchWorkspace, P4TaskWorkspace, ResearchRunEventDraft,
    ResearchRunStatus, ResolveP4ConflictCommand,
};
use serde_json::{json, Value};
use std::collections::BTreeSet;
use uuid::Uuid;

impl ApplicationService {
    pub async fn read_p4_match_workspace(
        &self,
        match_id: Uuid,
    ) -> ApplicationResult<P4MatchWorkspace> {
        Ok(self
            .active_store()
            .await?
            .read_p4_match_workspace(match_id)
            .await?)
    }

    pub async fn read_p4_task_workspace(
        &self,
        task_id: Uuid,
    ) -> ApplicationResult<P4TaskWorkspace> {
        Ok(self
            .active_store()
            .await?
            .read_p4_task_workspace(task_id)
            .await?)
    }

    pub async fn resolve_p4_conflict(
        &self,
        command: ResolveP4ConflictCommand,
    ) -> ApplicationResult<P4TaskWorkspace> {
        let store = self.active_store().await?;
        let workspace = store.read_p4_task_workspace(command.task_id).await?;
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
                reconcile_p4_task_after_manual_decision(&store, task.id, research_run_id).await?;
                return Ok(store.read_p4_task_workspace(task.id).await?);
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
        if Utc::now() >= task.data_cutoff_at {
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
            .ok_or_else(|| {
                ApplicationError::Validation("未找到与冲突对应的阻断路由".to_string())
            })?;
        let (selected_evidence_ids, selected_value, verification_state, route_status, reason) =
            match command.decision_kind {
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
        store
            .append_p4_manual_route_override(&P4ManualRouteOverrideDraft {
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
            })
            .await?;

        reconcile_p4_task_after_manual_decision(&store, task.id, research_run_id).await?;
        Ok(store.read_p4_task_workspace(task.id).await?)
    }
}

async fn reconcile_p4_task_after_manual_decision(
    store: &PersistenceStore,
    task_id: Uuid,
    research_run_id: Uuid,
) -> ApplicationResult<()> {
    let route_readiness = store.p4_route_readiness(task_id).await?;
    if route_readiness.ready {
        store
            .record_research_run_event(&ResearchRunEventDraft {
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
            let current = store.read_p4_freeze_task(task_id).await?;
            let recovered = match current.state {
                P4FreezeTaskState::ResearchPartial | P4FreezeTaskState::Blocked => {
                    match store
                        .transition_p4_freeze_task(&P4FreezeTaskTransition {
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
                        })
                        .await
                    {
                        Ok(task) => task,
                        Err(error) => {
                            let latest = store.read_p4_freeze_task(task_id).await?;
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

            match finalize_successful_research(store, &recovered).await {
                Ok(_) => return Ok(()),
                Err(error) => {
                    let latest = store.read_p4_freeze_task(task_id).await?;
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

    let current = store.read_p4_freeze_task(task_id).await?;
    match current.state {
        P4FreezeTaskState::ResearchPartial => {
            match store
                .transition_p4_freeze_task(&P4FreezeTaskTransition {
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
                })
                .await
            {
                Ok(_) => Ok(()),
                Err(error) => {
                    let latest = store.read_p4_freeze_task(task_id).await?;
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
