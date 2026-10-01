use super::{
    execute,
    lineups::selected_lineup,
    report::{readiness_check, summarize},
};
use crate::ports::PortErrorKind;
use crate::use_cases::prediction::shared::audit::{build_prediction_input_manifest, sha256_value};
use crate::use_cases::prediction::tests::{probe_registry, Probe};
use crate::{ApplicationError, StoredMatchPredictionCommand};
use chrono::{DateTime, Utc};
use football_domain::{
    CompetitionKind, LineupRecord, MatchLineupChain, MatchLineupTeamChain, MatchRecord,
    MatchStatus, PredictionReadinessCheckStatus as Status, PredictionReadinessLevel as Level,
    PreparedMatchPredictionInput, PREDICTION_INPUT_AUDIT_VERSION,
};
use serde_json::{json, Value};
use std::sync::Arc;
use uuid::Uuid;

const CALLS: [&str; 6] = [
    "read_match",
    "read_chain_at",
    "scope",
    "route",
    "model_supports",
    "prepare_input_at",
];

fn fixture() -> (Arc<Probe>, StoredMatchPredictionCommand) {
    let cutoff = DateTime::parse_from_rfc3339("2026-10-01T12:00:00.123456789Z")
        .unwrap()
        .with_timezone(&Utc);
    let record = MatchRecord {
        id: Uuid::from_u128(1),
        external_key: "R8-READINESS".into(),
        competition_id: Some(Uuid::from_u128(2)),
        competition_name: Some("Test".into()),
        season_id: Some(Uuid::from_u128(3)),
        stage_id: Some(Uuid::from_u128(4)),
        round_id: None,
        home_team_id: Uuid::from_u128(5),
        home_team_name: "Home".into(),
        away_team_id: Uuid::from_u128(6),
        away_team_name: "Away".into(),
        kickoff_time: cutoff + chrono::Duration::hours(1),
        status: MatchStatus::Scheduled,
        venue: None,
    };
    let side = |team_id, team_name: &str, team_side: &str, lineup_id| {
        // Synthetic orchestration data; the audit consumes prior lineup validation, not a model result.
        let players = (0..11).map(|index| json!({
            "player_id": Uuid::from_u128(100 + index), "player_name": format!("Player {index}"),
            "position_code": if index == 0 { "GK" } else { "CM" }, "role_code": "test_role",
            "role_origin": "player_position_default", "is_starter": true, "sequence_no": index,
            "availability_status": "available", "membership_override": false, "source_urls": []
        })).collect::<Vec<_>>();
        let lineup: LineupRecord = serde_json::from_value(json!({
            "id": lineup_id, "match_id": record.id, "match_key": record.external_key,
            "team_id": team_id, "team_name": team_name, "lineup_type": "confirmed",
            "snapshot_type": "T-1h", "captured_at": cutoff, "status": "active",
            "source_urls": [], "model_validation_status": "valid", "model_eligible": true,
            "validation_errors": [], "validation_warnings": [], "player_count": 11,
            "starter_count": 11, "players": players
        }))
        .unwrap();
        MatchLineupTeamChain {
            team_id,
            team_name: team_name.into(),
            team_side: team_side.into(),
            selected_lineup_id: Some(lineup_id),
            versions: vec![lineup],
            blocking_issues: vec![],
        }
    };
    let chain = MatchLineupChain {
        match_record: record.clone(),
        snapshot_type: "T-1h".into(),
        data_window_start_time: Some(cutoff - chrono::Duration::hours(1)),
        data_cutoff_time: cutoff,
        home: side(record.home_team_id, "Home", "home", Uuid::from_u128(7)),
        away: side(record.away_team_id, "Away", "away", Uuid::from_u128(8)),
        ready_for_model: true,
        blocking_issues: vec![],
    };
    let prepared = PreparedMatchPredictionInput {
        match_record: record.clone(),
        competition_kind: CompetitionKind::League,
        snapshot_type: "T-1h".into(),
        match_input: json!({"match_id": record.external_key, "feature_quality_score": 0.65, "preparation_version": "test", "team_a": {"strength": 12}, "team_b": {"strength": 11}}),
        data_quality: json!({"home": {"team_features": {"history_match_count": 5}}, "away": {"team_features": {"history_match_count": 5}}}),
    };
    let port = Arc::new(Probe::new());
    {
        let mut state = port.state.lock().unwrap();
        state.scope_kind = Some(CompetitionKind::League);
        state.match_record = Some(record.clone());
        state.match_chain = Some(chain);
        state.prepared_input = Some(prepared);
    }
    (
        port,
        StoredMatchPredictionCommand {
            match_id: record.id,
            snapshot_type: "T-1h".into(),
            model_family: "P4_LEAGUE".into(),
            explicit_rule_package_id: Some(Uuid::from_u128(10)),
        },
    )
}

#[tokio::test]
async fn formal_report_preserves_one_clock_order_route_manifest_and_read_only_calls() {
    let (port, command) = fixture();
    let report = execute(port.as_ref(), &probe_registry(&port), command.clone())
        .await
        .unwrap();
    assert_eq!(port.calls(), CALLS);
    assert_eq!(report.level, Level::FormalReady);
    assert_eq!(report.score, 100);
    assert!(report.can_run_formal && report.can_run_shadow);
    assert!(report.blockers.is_empty() && report.warnings.is_empty());
    assert_eq!(report.audit_version, PREDICTION_INPUT_AUDIT_VERSION);
    assert_eq!(report.match_id, command.match_id);
    assert_eq!(report.match_key, "R8-READINESS");
    assert_eq!(report.snapshot_type, command.snapshot_type);
    assert_eq!(report.model_family, command.model_family);
    assert_eq!(
        report
            .checks
            .iter()
            .map(|check| check.code.as_str())
            .collect::<Vec<_>>(),
        [
            "match_identity",
            "data_window",
            "home_lineup",
            "away_lineup",
            "starting_goalkeepers",
            "starter_context",
            "model_route",
            "team_history",
            "model_input"
        ]
    );
    let state = port.state.lock().unwrap();
    assert_eq!(
        state.chain_requests,
        [(command.match_id, "T-1h".into(), report.assessed_at)]
    );
    assert_eq!(
        state.input_requests,
        [(
            command.match_id,
            "T-1h".into(),
            "p4".into(),
            report.assessed_at
        )]
    );
    assert_eq!(
        report.data_cutoff_at,
        Some(state.match_chain.as_ref().unwrap().data_cutoff_time)
    );
    let prepared = state.prepared_input.as_ref().unwrap();
    let manifest = build_prediction_input_manifest(
        &prepared.match_input,
        &prepared.data_quality,
        state.match_record.as_ref().unwrap(),
        "T-1h",
        report.route_identity.as_ref(),
    );
    assert_eq!(
        report.input_manifest_sha256,
        Some(sha256_value(&manifest).unwrap())
    );
    assert_eq!(report.input_manifest, Some(manifest));
    assert_eq!(
        report.route_identity.as_ref().unwrap()["rule_package_id"],
        json!(command.explicit_rule_package_id)
    );
    assert_eq!(
        report.route_identity.as_ref().unwrap()["model_id"],
        "p4_league"
    );
}

#[tokio::test]
async fn history_and_quality_thresholds_preserve_scores_and_mode_permissions() {
    for (history, quality, expected_level, expected_score) in [
        (5, 0.65, Level::FormalReady, 100),
        (5, 2.0, Level::FormalReady, 100),
        (4, 0.65, Level::ReadyWithWarnings, 96),
        (5, 0.40, Level::ReadyWithWarnings, 98),
        (5, 0.399, Level::ShadowOnly, 96),
        (5, -1.0, Level::ShadowOnly, 96),
        (0, 0.65, Level::ShadowOnly, 92),
    ] {
        let (port, command) = fixture();
        {
            let mut state = port.state.lock().unwrap();
            let prepared = state.prepared_input.as_mut().unwrap();
            prepared.data_quality["home"]["team_features"]["history_match_count"] = json!(history);
            prepared.match_input["feature_quality_score"] = json!(quality);
        }
        let report = execute(port.as_ref(), &probe_registry(&port), command)
            .await
            .unwrap();
        assert_eq!(
            (report.level, report.score),
            (expected_level, expected_score)
        );
        assert_eq!(report.can_run_formal, expected_level.can_run_formal());
        assert_eq!(report.can_run_shadow, expected_level.can_run_shadow());
        assert_eq!(port.calls(), CALLS);
    }
}

#[tokio::test]
async fn lineup_selection_goalkeepers_starter_context_and_identity_keep_blocking_priority() {
    for problem in [
        "expected",
        "missing_selected",
        "two_goalkeepers",
        "missing_position",
        "unavailable",
        "missing_role",
        "unknown_status",
        "same_team",
    ] {
        let (port, command) = fixture();
        {
            let mut state = port.state.lock().unwrap();
            if problem == "same_team" {
                let record = state.match_record.as_mut().unwrap();
                record.away_team_id = record.home_team_id;
            } else {
                let chain = state.match_chain.as_mut().unwrap();
                let lineup = &mut chain.home.versions[0];
                match problem {
                    "expected" => {
                        lineup.lineup_type = serde_json::from_value(json!("expected")).unwrap()
                    }
                    "missing_selected" => {
                        chain.home.selected_lineup_id = Some(Uuid::from_u128(999))
                    }
                    "two_goalkeepers" => lineup.players[1].position_code = Some("gk".into()),
                    "missing_position" => lineup.players[1].position_code = Some(" ".into()),
                    "unavailable" => {
                        lineup.players[1].availability_status =
                            Some(football_domain::AvailabilityStatus::Injured)
                    }
                    "missing_role" => lineup.players[1].role_code = None,
                    "unknown_status" => lineup.players[1].availability_status = None,
                    _ => unreachable!(),
                }
                if problem == "missing_selected" {
                    assert!(selected_lineup(&chain.home).is_none());
                }
            }
            // Shadow reasons must never override a blocking check.
            state.prepared_input.as_mut().unwrap().match_input["feature_quality_score"] =
                json!(0.3);
        }
        let report = execute(port.as_ref(), &probe_registry(&port), command)
            .await
            .unwrap();
        let blocked = !["expected", "missing_role", "unknown_status"].contains(&problem);
        assert_eq!(
            report.level,
            if blocked {
                Level::Blocked
            } else {
                Level::ShadowOnly
            }
        );
        assert!(!report.can_run_formal);
        assert_eq!(report.can_run_shadow, !blocked);
        assert_eq!(port.calls(), CALLS);
        if problem == "missing_role" {
            assert_eq!(report.checks[5].metadata["missing_role_count"], 1);
        }
        if problem == "unknown_status" {
            assert_eq!(report.checks[5].metadata["missing_availability_count"], 1);
        }
    }
}

#[tokio::test]
async fn unavailable_window_input_and_missing_route_become_reports_without_retry() {
    for (failure, kind, prepares) in [
        ("read_chain_at", PortErrorKind::InvalidState, false),
        ("prepare_input_at", PortErrorKind::InvalidState, true),
        ("route", PortErrorKind::NotFound, true),
    ] {
        let (port, command) = fixture();
        {
            let mut state = port.state.lock().unwrap();
            state.failure = Some(failure);
            state.failure_kind = Some(kind);
        }
        let report = execute(port.as_ref(), &probe_registry(&port), command)
            .await
            .unwrap();
        assert_eq!(report.level, Level::Blocked);
        assert!(!report.can_run_formal && !report.can_run_shadow);
        let calls = port.calls();
        assert_eq!(calls.iter().filter(|&&call| call == failure).count(), 1);
        assert_eq!(calls.contains(&"prepare_input_at"), prepares);
        assert_eq!(report.input_manifest.is_some(), failure == "route");
        assert_eq!(report.data_cutoff_at.is_some(), failure != "read_chain_at");
        if failure == "read_chain_at" {
            assert_eq!(
                calls,
                [
                    "read_match",
                    "read_chain_at",
                    "scope",
                    "route",
                    "model_supports"
                ]
            );
        }
        if failure == "route" {
            assert_eq!(
                calls,
                [
                    "read_match",
                    "read_chain_at",
                    "scope",
                    "route",
                    "prepare_input_at"
                ]
            );
        }
    }
    let (port, command) = fixture();
    port.state
        .lock()
        .unwrap()
        .match_chain
        .as_mut()
        .unwrap()
        .ready_for_model = false;
    let report = execute(port.as_ref(), &probe_registry(&port), command)
        .await
        .unwrap();
    assert_eq!(report.level, Level::Blocked);
    assert!(!port.calls().contains(&"prepare_input_at"));
    assert!(report.input_manifest.is_none());
}

#[tokio::test]
async fn port_unavailable_errors_stop_at_the_original_read_boundary() {
    for (index, &failure) in CALLS
        .iter()
        .enumerate()
        .filter(|(_, call)| **call != "model_supports")
    {
        let (port, command) = fixture();
        port.fail_at(Some(failure));
        let result = execute(port.as_ref(), &probe_registry(&port), command).await;
        assert!(
            matches!(result, Err(ApplicationError::Port(error)) if error.kind == PortErrorKind::Unavailable && error.message == format!("injected {failure}"))
        );
        assert_eq!(port.calls(), CALLS[..=index]);
    }
}

#[tokio::test]
async fn route_snapshot_scope_and_model_support_failures_remain_blocked_reports() {
    for problem in ["snapshot", "scope", "supports"] {
        let (port, mut command) = fixture();
        match problem {
            "snapshot" => command.snapshot_type = "T-6h".into(),
            "scope" => {
                command.explicit_rule_package_id = None;
                port.state.lock().unwrap().scope_kind = Some(CompetitionKind::Custom);
            }
            "supports" => port.fail_at(Some("model_supports")),
            _ => unreachable!(),
        }
        let report = execute(port.as_ref(), &probe_registry(&port), command)
            .await
            .unwrap();
        assert_eq!(report.level, Level::Blocked);
        assert!(report.route_identity.is_none());
        assert!(report
            .blockers
            .iter()
            .any(|reason| reason.starts_with("模型与规则路由：")));
        assert!(report.input_manifest.is_some());
        if problem != "supports" {
            assert!(!port.calls().contains(&"model_supports"));
        }
        assert!(!port
            .calls()
            .iter()
            .any(|call| ["predict", "save_run", "enqueue"].contains(call)));
    }
}

#[tokio::test]
async fn invalid_family_and_unregistered_selection_reject_before_read_io() {
    let (port, mut command) = fixture();
    command.model_family = "unknown".into();
    assert!(matches!(
        execute(port.as_ref(), &probe_registry(&port), command).await,
        Err(ApplicationError::Validation(_))
    ));
    assert!(port.calls().is_empty());
    let (port, command) = fixture();
    assert!(matches!(
        execute(
            port.as_ref(),
            &crate::model_registry::ModelRegistry::new(),
            command
        )
        .await,
        Err(ApplicationError::ModelNotFound(_))
    ));
    assert!(port.calls().is_empty());
}

#[test]
fn report_classification_preserves_labels_reason_order_dedup_and_score_caps() {
    let warning = readiness_check(
        ("warning", "警告"),
        Status::Warning,
        80,
        99,
        "摘要",
        vec![],
        Value::Null,
    );
    assert_eq!(warning.score, 80);
    let passed = readiness_check(
        ("pass", "通过"),
        Status::Passed,
        80,
        80,
        "摘要",
        vec![],
        Value::Null,
    );
    let summary = summarize(
        &[warning.clone(), passed],
        &["警告：摘要".into(), "影子原因".into(), "影子原因".into()],
    );
    assert_eq!(summary.score, 100);
    assert_eq!(summary.level, Level::ShadowOnly);
    assert_eq!(summary.warnings, ["警告：摘要", "影子原因"]);
    let blocked = readiness_check(
        ("block", "阻断"),
        Status::Blocked,
        10,
        0,
        "备用摘要",
        vec!["原因一".into(), "原因二".into()],
        Value::Null,
    );
    let summary = summarize(&[warning, blocked], &["影子原因".into()]);
    assert_eq!(summary.level, Level::Blocked);
    assert_eq!(summary.blockers, ["阻断：原因一", "阻断：原因二"]);
    assert_eq!(summary.warnings, ["警告：摘要", "影子原因"]);
}
