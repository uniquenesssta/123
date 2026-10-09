use crate::ports::prediction::PredictionWorkflowPort;
use crate::ApplicationResult;
use football_domain::P4MatchWorkspace;
use uuid::Uuid;

pub(crate) async fn execute<P: PredictionWorkflowPort + ?Sized>(
    port: &P,
    match_id: Uuid,
) -> ApplicationResult<P4MatchWorkspace> {
    Ok(port.read_match_workspace(match_id).await?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ports::PortErrorKind;
    use crate::services::p4_orchestration::tests::task_fixture;
    use crate::use_cases::prediction::tests::Probe;
    use crate::ApplicationError;
    use football_domain::P4FreezeTaskState;

    #[tokio::test]
    async fn match_workspace_preserves_identity_nullable_competition_and_task_order_without_writes()
    {
        for competition_name in [None, Some("原赛事名称".to_string())] {
            let probe = Probe::new();
            let task = task_fixture(P4FreezeTaskState::Frozen);
            let mut second = task.clone();
            second.id = Uuid::from_u128(29);
            second.state = P4FreezeTaskState::Planned;
            let workspace = P4MatchWorkspace {
                match_id: task.match_id,
                match_key: " raw-match-key ".into(),
                home_team_name: "主队".into(),
                away_team_name: "客队".into(),
                kickoff_at: task.kickoff_at,
                competition_name,
                tasks: vec![task, second],
            };
            let expected = serde_json::to_value(&workspace).unwrap();
            let match_id = workspace.match_id;
            probe.state.lock().unwrap().match_workspace = Some(workspace);
            assert_eq!(
                serde_json::to_value(execute(&probe, match_id).await.unwrap()).unwrap(),
                expected
            );
            assert_eq!(probe.calls(), ["match_workspace"]);
            let state = probe.state.lock().unwrap();
            assert_eq!(state.match_workspace_requests, [match_id]);
            assert!(state.task_workspace_requests.is_empty());
            assert!(state.transitions.is_empty() && state.enqueues.is_empty());
        }
    }

    #[tokio::test]
    async fn match_workspace_preserves_all_port_errors_without_retry_or_fallback() {
        for kind in [
            PortErrorKind::Unavailable,
            PortErrorKind::NotFound,
            PortErrorKind::Conflict,
            PortErrorKind::InvalidState,
            PortErrorKind::Serialization,
            PortErrorKind::Infrastructure,
        ] {
            let probe = Probe::new();
            probe.fail_at(Some("match_workspace"));
            probe.state.lock().unwrap().failure_kind = Some(kind);
            let ApplicationError::Port(error) = execute(&probe, Uuid::nil()).await.unwrap_err()
            else {
                panic!("workspace error must retain its original boundary");
            };
            assert_eq!(error.kind, kind);
            assert_eq!(error.message, "injected match_workspace");
            assert_eq!(probe.calls(), ["match_workspace"]);
        }
    }
}
