use crate::ports::prediction::PredictionWorkflowPort;
use crate::ApplicationResult;
use football_domain::P4TaskWorkspace;
use uuid::Uuid;

pub(crate) async fn execute<P: PredictionWorkflowPort + ?Sized>(
    port: &P,
    task_id: Uuid,
) -> ApplicationResult<P4TaskWorkspace> {
    Ok(port.read_task_workspace(task_id).await?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ports::PortErrorKind;
    use crate::services::p4_orchestration::tests::task_fixture;
    use crate::use_cases::prediction::tests::Probe;
    use crate::ApplicationError;
    use football_domain::P4FreezeTaskState;
    use serde_json::json;

    #[tokio::test]
    async fn task_workspace_preserves_empty_and_populated_views_in_progressed_and_terminal_states()
    {
        for state in [
            P4FreezeTaskState::Planned,
            P4FreezeTaskState::ResearchQueued,
            P4FreezeTaskState::Frozen,
            P4FreezeTaskState::Failed,
            P4FreezeTaskState::Cancelled,
        ] {
            for populated in [false, true] {
                let probe = Probe::new();
                let mut task = task_fixture(state);
                if !populated {
                    task.research_run_id = None;
                }
                let mut value = json!({
                    "task": task,
                    "readiness": {"task_id":task.id,"ready":false,"research_status":null,
                        "requested_fact_count":1,"routed_fact_count":0,"missing_fact_count":0,
                        "ignored_fact_count":0,"blocked_fact_count":0,"blockers":["原始阻断原因"]},
                    "events":[],"research_run":null,"routes":[],"evidence":[],"conflicts":[],"snapshot":null
                });
                if populated {
                    let evidence_id = Uuid::from_u128(50);
                    value["research_run"] = json!({"id":task.research_run_id,"status":"failed","attempt_count":3,
                        "response_id":"response","model_id":"model","error_category":"provider",
                        "error_message":"原始错误","created_at":task.created_at,"started_at":task.created_at,
                        "finished_at":task.updated_at});
                    value["events"] = json!([{"id":Uuid::from_u128(51),"task_id":task.id,
                        "from_state":null,"to_state":state,"reason":"原事件","payload":{"preserve":true},
                        "occurred_at":task.updated_at}]);
                    value["routes"] = json!([{"route_key":"raw:route","field_key":"fact",
                        "target_module":"fixture","target_slot":"home","route_status":"routed",
                        "verification_state":"PROBABLE","selected_evidence_ids":[evidence_id],
                        "selected_value":{"raw":"保留"},"reason":"manual"}]);
                    value["evidence"] = json!([{"id":evidence_id,"field_key":"fact","entity_type":"team",
                        "entity_id":null,"value":{"raw":"保留"},"verification_state":"PROBABLE",
                        "source_tier":"official","source_url":"https://example.test/evidence",
                        "source_title":"来源","source_domain":"example.test","published_at":null,
                        "observed_at":task.created_at,"effective_at":null,"retrieved_at":task.updated_at,
                        "timezone":"UTC","conflict_group_id":null,"created_at":task.created_at}]);
                    value["conflicts"] = json!([{"id":Uuid::from_u128(52),"field_key":"fact",
                        "entity_type":"team","entity_id":null,"conflict_key":"raw:conflict","status":"resolved",
                        "evaluation_status":"auto_resolved","evidence_ids":[evidence_id],
                        "selected_evidence_ids":[evidence_id],"manual_decision_kind":"select_evidence",
                        "manual_decision_note":"原人工备注","manual_decision_at":task.updated_at,
                        "created_at":task.created_at}]);
                }
                let workspace: P4TaskWorkspace = serde_json::from_value(value).unwrap();
                let expected = serde_json::to_value(&workspace).unwrap();
                probe.state.lock().unwrap().task_workspace = Some(workspace);
                assert_eq!(
                    serde_json::to_value(execute(&probe, task.id).await.unwrap()).unwrap(),
                    expected
                );
                assert_eq!(probe.calls(), ["task_workspace"]);
                let recorded = probe.state.lock().unwrap();
                assert_eq!(recorded.task_workspace_requests, [task.id]);
                assert!(recorded.match_workspace_requests.is_empty());
                assert!(recorded.transitions.is_empty() && recorded.enqueues.is_empty());
            }
        }
    }

    #[tokio::test]
    async fn task_workspace_preserves_all_port_errors_without_partial_view_or_retry() {
        for kind in [
            PortErrorKind::Unavailable,
            PortErrorKind::NotFound,
            PortErrorKind::Conflict,
            PortErrorKind::InvalidState,
            PortErrorKind::Serialization,
            PortErrorKind::Infrastructure,
        ] {
            let probe = Probe::new();
            probe.fail_at(Some("task_workspace"));
            probe.state.lock().unwrap().failure_kind = Some(kind);
            let ApplicationError::Port(error) = execute(&probe, Uuid::nil()).await.unwrap_err()
            else {
                panic!("workspace error must retain its original boundary");
            };
            assert_eq!(error.kind, kind);
            assert_eq!(error.message, "injected task_workspace");
            assert_eq!(probe.calls(), ["task_workspace"]);
        }
    }
}
