use super::*;
use crate::ApplicationError;
use chrono::Utc;
use football_domain::PREDICTION_INPUT_AUDIT_VERSION;
use serde_json::{json, Value};
use uuid::Uuid;

#[test]
fn input_manifest_ignores_runtime_snapshot_identity() {
    let match_record = football_domain::MatchRecord {
        id: Uuid::nil(),
        external_key: "MATCH-1".to_string(),
        competition_id: None,
        competition_name: None,
        season_id: None,
        stage_id: None,
        round_id: None,
        home_team_id: Uuid::from_u128(1),
        home_team_name: "Home".to_string(),
        away_team_id: Uuid::from_u128(2),
        away_team_name: "Away".to_string(),
        kickoff_time: Utc::now(),
        status: football_domain::MatchStatus::Scheduled,
        venue: None,
    };
    let route = json!({"model_id": "p4_league", "parameter_version": "p1"});
    let first = json!({
        "snapshot": {"snapshot_id": Uuid::new_v4(), "type": "T-1h", "data_cutoff_time": "2026-07-22T10:00:00Z", "frozen_at": "2026-07-22T10:01:00Z"},
        "feature_snapshot_id": Uuid::new_v4(),
        "preparation_version": "v1",
        "feature_quality_score": 0.8,
        "team_a": {"team_id": Uuid::from_u128(1), "lineup": {"lineup_id": Uuid::from_u128(3), "player_contributions": [{"player_id": Uuid::from_u128(5), "calculation_version": "v1", "effective_contribution": 71.0}]}},
        "team_b": {"team_id": Uuid::from_u128(2), "lineup": {"lineup_id": Uuid::from_u128(4), "player_contributions": []}},
        "sources": [{"source_id": "database", "accessed_at": "2026-07-22T10:01:00Z"}]
    });
    let mut second = first.clone();
    second["snapshot"]["snapshot_id"] = json!(Uuid::new_v4());
    second["snapshot"]["frozen_at"] = json!("2026-07-22T10:02:00Z");
    second["feature_snapshot_id"] = json!(Uuid::new_v4());
    second["sources"][0]["accessed_at"] = json!("2026-07-22T10:02:00Z");
    let quality = json!({"home": {}, "away": {}});
    let first_manifest =
        build_prediction_input_manifest(&first, &quality, &match_record, "T-1h", Some(&route));
    let second_manifest =
        build_prediction_input_manifest(&second, &quality, &match_record, "T-1h", Some(&route));
    assert_eq!(
        sha256_value(&first_manifest).unwrap(),
        sha256_value(&second_manifest).unwrap()
    );

    let mut changed = second;
    changed["team_a"]["lineup"]["player_contributions"][0]["effective_contribution"] = json!(72.0);
    let changed_manifest =
        build_prediction_input_manifest(&changed, &quality, &match_record, "T-1h", Some(&route));
    assert_ne!(
        sha256_value(&first_manifest).unwrap(),
        sha256_value(&changed_manifest).unwrap()
    );
}

#[test]
fn audit_summary_rejects_modified_manifest() {
    let manifest = json!({"match": "A"});
    let manifest_sha = sha256_value(&manifest).unwrap();
    let mut input = json!({
        "input_audit": {
            "audit_version": PREDICTION_INPUT_AUDIT_VERSION,
            "readiness": {"level": "formal_ready", "score": 100},
            "manifest": manifest,
            "manifest_sha256": manifest_sha
        }
    });
    assert!(prediction_input_audit_summary(&input, "input-hash")
        .unwrap()
        .is_some());
    input["input_audit"]["manifest"]["match"] = json!("B");
    assert!(prediction_input_audit_summary(&input, "input-hash").is_err());
}

fn fixture() -> (
    football_domain::PreparedMatchPredictionInput,
    football_domain::MatchPredictionReadiness,
) {
    use football_domain::{CompetitionKind, MatchRecord, MatchStatus, PredictionReadinessLevel};
    let assessed_at = chrono::DateTime::parse_from_rfc3339("2026-10-01T12:00:00.123456789Z")
        .unwrap()
        .with_timezone(&Utc);
    let record = MatchRecord {
        id: Uuid::from_u128(1),
        external_key: "R8-清单".into(),
        competition_id: None,
        competition_name: None,
        season_id: Some(Uuid::from_u128(2)),
        stage_id: None,
        round_id: None,
        home_team_id: Uuid::from_u128(3),
        home_team_name: "主队".into(),
        away_team_id: Uuid::from_u128(4),
        away_team_name: "客队".into(),
        kickoff_time: chrono::DateTime::parse_from_rfc3339("2026-10-01T13:00:00Z")
            .unwrap()
            .with_timezone(&Utc),
        status: MatchStatus::Scheduled,
        venue: None,
    };
    let prepared = football_domain::PreparedMatchPredictionInput {
        match_record: record,
        competition_kind: CompetitionKind::League,
        snapshot_type: "T-1h".into(),
        match_input: json!({
            "match_id": "R8-清单", "feature_snapshot_id": Uuid::from_u128(9), "input_audit": {"previous": true},
            "snapshot": {"snapshot_id": Uuid::from_u128(10), "frozen_at": "2026-10-01T12:01:00Z", "type": "T-1h", "data_cutoff_time": "2026-10-01T12:00:00.123456789Z"},
            "team_a": {"lineup_id": Uuid::from_u128(5), "contributions": [{"player_id": Uuid::from_u128(6), "effective": 71.0}]},
            "team_b": {"strength": 12},
            "sources": [{"source_id": "database", "url": "https://example.invalid/事实", "accessed_at": "2026-10-01T12:02:00Z"}]
        }),
        data_quality: json!({"home": {"history_match_count": 5}, "away": {"history_match_count": 6}}),
    };
    let route = json!({"model_id": "p4_league", "rule_package_id": Uuid::from_u128(7)});
    let manifest = build_prediction_input_manifest(
        &prepared.match_input,
        &prepared.data_quality,
        &prepared.match_record,
        &prepared.snapshot_type,
        Some(&route),
    );
    let report = football_domain::MatchPredictionReadiness {
        audit_version: PREDICTION_INPUT_AUDIT_VERSION.into(),
        match_id: prepared.match_record.id,
        match_key: prepared.match_record.external_key.clone(),
        snapshot_type: prepared.snapshot_type.clone(),
        model_family: "p4".into(),
        assessed_at,
        data_cutoff_at: Some(assessed_at),
        level: PredictionReadinessLevel::ShadowOnly,
        score: 42,
        can_run_formal: false,
        can_run_shadow: true,
        blockers: vec![],
        warnings: vec!["样本不足".into()],
        checks: vec![],
        input_manifest_sha256: Some(sha256_value(&manifest).unwrap()),
        input_manifest: Some(manifest),
        route_identity: Some(route),
    };
    (prepared, report)
}

#[test]
fn fixed_manifest_preserves_original_shape_serialization_and_fingerprint() {
    let (prepared, report) = fixture();
    let original = prepared.match_input.clone();
    let manifest = build_prediction_input_manifest(
        &original,
        &prepared.data_quality,
        &prepared.match_record,
        "T-1h",
        report.route_identity.as_ref(),
    );
    let expected = json!({
        "audit_version": "prematch-input-audit-v1",
        "match": {"database_match_id": Uuid::from_u128(1), "match_key": "R8-清单", "competition_id": null, "season_id": Uuid::from_u128(2), "stage_id": null, "home_team_id": Uuid::from_u128(3), "away_team_id": Uuid::from_u128(4), "kickoff_time": "2026-10-01T13:00:00Z"},
        "snapshot_type": "T-1h", "route_identity": {"model_id": "p4_league", "rule_package_id": Uuid::from_u128(7)},
        "model_input": {"match_id": "R8-清单", "snapshot": {"type": "T-1h", "data_cutoff_time": "2026-10-01T12:00:00.123456789Z"}, "team_a": {"lineup_id": Uuid::from_u128(5), "contributions": [{"player_id": Uuid::from_u128(6), "effective": 71.0}]}, "team_b": {"strength": 12}, "sources": [{"source_id": "database", "url": "https://example.invalid/事实"}]},
        "data_quality": {"home": {"history_match_count": 5}, "away": {"history_match_count": 6}}
    });
    assert_eq!(manifest, expected);
    assert_eq!(prepared.match_input, original);
    // Frozen public platform vector, not private P4/P7 model probability evidence.
    assert_eq!(
        sha256_value(&manifest).unwrap(),
        "178afe68af4d0cb8ba9341a7f5f47ec3b89c4c2b9ceaafd0e6615db3a9a0fb87"
    );
    assert_eq!(
        sha256_value(&json!({})).unwrap(),
        "44136fa355b3678a1146ad16f7e8649e94fb4fc21fe77e8310c060f61caaff8a"
    );
    let reordered: Value = serde_json::from_str(r#"{"z":1,"a":{"y":2,"b":3}}"#).unwrap();
    assert_eq!(
        sha256_value(&reordered).unwrap(),
        sha256_value(&json!({"a": {"b": 3, "y": 2}, "z": 1})).unwrap()
    );
}

#[test]
fn runtime_exclusion_is_limited_to_original_documented_locations() {
    let (prepared, report) = fixture();
    let fingerprint = |input: &Value| {
        sha256_value(&build_prediction_input_manifest(
            input,
            &prepared.data_quality,
            &prepared.match_record,
            "T-1h",
            report.route_identity.as_ref(),
        ))
        .unwrap()
    };
    let mut runtime = prepared.match_input.clone();
    runtime["feature_snapshot_id"] = json!("changed");
    runtime["snapshot"]["snapshot_id"] = json!("changed");
    runtime["snapshot"]["frozen_at"] = json!("changed");
    runtime["sources"][0]["accessed_at"] = json!("changed");
    runtime["input_audit"] = json!({"untrusted": true});
    assert_eq!(fingerprint(&runtime), fingerprint(&prepared.match_input));
    assert_ne!(
        sha256_value(&runtime).unwrap(),
        sha256_value(&prepared.match_input).unwrap()
    );
    for (parent, field) in [
        ("team_a", "feature_snapshot_id"),
        ("team_a", "input_audit"),
        ("team_b", "accessed_at"),
        ("snapshot", "data_cutoff_time"),
    ] {
        let mut semantic = runtime.clone();
        semantic[parent][field] = json!("retained");
        assert_ne!(fingerprint(&semantic), fingerprint(&runtime));
    }
    runtime["sources"] =
        json!([null, "plain", {"accessed_at": "ignored", "metadata": {"accessed_at": "retained"}}]);
    let manifest = build_prediction_input_manifest(
        &runtime,
        &prepared.data_quality,
        &prepared.match_record,
        "T-1h",
        None,
    );
    assert_eq!(
        manifest["model_input"]["sources"],
        json!([null, "plain", {"metadata": {"accessed_at": "retained"}}])
    );
}

#[test]
fn semantic_input_quality_route_match_and_window_changes_alter_fingerprint() {
    let (prepared, report) = fixture();
    for change in [
        "contribution",
        "lineup",
        "cutoff",
        "source",
        "source_order",
        "quality",
        "route",
        "match",
        "kickoff",
        "season",
        "snapshot",
    ] {
        let mut changed = prepared.clone();
        let mut route = report.route_identity.clone();
        match change {
            "contribution" => {
                changed.match_input["team_a"]["contributions"][0]["effective"] = json!(72.0)
            }
            "lineup" => changed.match_input["team_a"]["lineup_id"] = json!(Uuid::from_u128(99)),
            "cutoff" => {
                changed.match_input["snapshot"]["data_cutoff_time"] =
                    json!("2026-10-01T12:00:00.123456788Z")
            }
            "source" => changed.match_input["sources"][0]["url"] = json!("different"),
            "source_order" => {
                changed.match_input["sources"] =
                    json!([{"source_id": "second"}, {"source_id": "database"}])
            }
            "quality" => changed.data_quality["home"]["history_match_count"] = json!(4),
            "route" => route.as_mut().unwrap()["rule_package_id"] = json!(Uuid::from_u128(99)),
            "match" => changed.match_record.home_team_id = Uuid::from_u128(99),
            "kickoff" => changed.match_record.kickoff_time += chrono::Duration::nanoseconds(1),
            "season" => changed.match_record.season_id = None,
            "snapshot" => changed.snapshot_type = "T-6h".into(),
            _ => unreachable!(),
        }
        let manifest = build_prediction_input_manifest(
            &changed.match_input,
            &changed.data_quality,
            &changed.match_record,
            &changed.snapshot_type,
            route.as_ref(),
        );
        assert_ne!(
            sha256_value(&manifest).unwrap(),
            report.input_manifest_sha256.clone().unwrap(),
            "{change}"
        );
    }
    let first = build_prediction_input_manifest(
        &json!({"sources":[{"source_id":"a"},{"source_id":"b"}]}),
        &prepared.data_quality,
        &prepared.match_record,
        "T-1h",
        None,
    );
    let second = build_prediction_input_manifest(
        &json!({"sources":[{"source_id":"b"},{"source_id":"a"}]}),
        &prepared.data_quality,
        &prepared.match_record,
        "T-1h",
        None,
    );
    assert_ne!(
        sha256_value(&first).unwrap(),
        sha256_value(&second).unwrap()
    );
}

#[test]
fn non_object_input_and_null_absence_preserve_original_manifest_semantics() {
    let (prepared, _) = fixture();
    for input in [
        Value::Null,
        json!([]),
        json!(7),
        json!("literal"),
        json!({"snapshot": null, "sources": "literal"}),
    ] {
        let before = input.clone();
        let manifest = build_prediction_input_manifest(
            &input,
            &prepared.data_quality,
            &prepared.match_record,
            "T-1h",
            None,
        );
        assert_eq!(manifest["model_input"], before);
        assert_eq!(input, before);
        assert!(manifest["route_identity"].is_null());
    }
    let absent = build_prediction_input_manifest(
        &json!({}),
        &prepared.data_quality,
        &prepared.match_record,
        "T-1h",
        None,
    );
    let null = build_prediction_input_manifest(
        &json!({"snapshot":null}),
        &prepared.data_quality,
        &prepared.match_record,
        "T-1h",
        None,
    );
    assert_ne!(sha256_value(&absent).unwrap(), sha256_value(&null).unwrap());
}

#[test]
fn prepared_verification_preserves_hash_requirement_runtime_exclusion_and_drift_errors() {
    let (mut prepared, mut report) = fixture();
    verify_prepared_input_matches_readiness(&prepared, &report).unwrap();
    prepared.match_input["feature_snapshot_id"] = json!("different");
    verify_prepared_input_matches_readiness(&prepared, &report).unwrap();
    // Verification uses the assessed hash; attachment separately requires the assessed manifest.
    report.input_manifest = None;
    verify_prepared_input_matches_readiness(&prepared, &report).unwrap();
    prepared.data_quality["home"]["history_match_count"] = json!(1);
    assert!(
        matches!(verify_prepared_input_matches_readiness(&prepared, &report), Err(ApplicationError::Validation(message)) if message == "赛前数据在完整度检查后发生变化，请重新检查完整度再执行推演")
    );
    report.input_manifest_sha256 = None;
    assert!(
        matches!(verify_prepared_input_matches_readiness(&prepared, &report), Err(ApplicationError::Validation(message)) if message == "完整度门禁没有生成输入指纹，禁止执行推演")
    );
}

#[test]
fn audit_attachment_preserves_payload_and_failure_priority() {
    let (prepared, report) = fixture();
    let mut input = prepared.match_input.clone();
    attach_prediction_input_audit(&mut input, &report).unwrap();
    let expected = json!({"audit_version": report.audit_version, "assessed_at": report.assessed_at, "readiness": {"level": report.level.as_str(), "score": report.score, "can_run_formal": report.can_run_formal, "can_run_shadow": report.can_run_shadow, "blockers": report.blockers, "warnings": report.warnings, "checks": report.checks}, "manifest": report.input_manifest, "manifest_sha256": report.input_manifest_sha256});
    assert_eq!(input["input_audit"], expected);
    let mut original = prepared.match_input.clone();
    original.as_object_mut().unwrap().remove("input_audit");
    input.as_object_mut().unwrap().remove("input_audit");
    assert_eq!(input, original);
    for (missing_manifest, missing_hash, message) in [
        (true, true, "完整度门禁没有生成输入清单，禁止执行推演"),
        (false, true, "完整度门禁没有生成输入指纹，禁止执行推演"),
        (false, false, "模型输入必须是 JSON 对象"),
    ] {
        let mut report = report.clone();
        if missing_manifest {
            report.input_manifest = None;
        }
        if missing_hash {
            report.input_manifest_sha256 = None;
        }
        let mut input = json!([]);
        let error = attach_prediction_input_audit(&mut input, &report).unwrap_err();
        let expected_validation = missing_manifest || missing_hash;
        assert!(match error {
            ApplicationError::Validation(actual) => expected_validation && actual == message,
            ApplicationError::Model(actual) => !expected_validation && actual == message,
            _ => false,
        });
        assert_eq!(input, json!([]));
    }
}

#[test]
fn audit_summary_preserves_field_errors_fallbacks_and_untrimmed_identity() {
    assert!(prediction_input_audit_summary(&json!({}), "input-sha")
        .unwrap()
        .is_none());
    assert!(
        matches!(prediction_input_audit_summary(&json!({"input_audit": null}), "input-sha"), Err(ApplicationError::Model(message)) if message == "input_audit.audit_version 不能为空")
    );
    let (prepared, report) = fixture();
    let mut input = prepared.match_input.clone();
    attach_prediction_input_audit(&mut input, &report).unwrap();
    for (field, value, message) in [
        (
            "audit_version",
            json!(" "),
            "input_audit.audit_version 不能为空",
        ),
        (
            "audit_version",
            json!(7),
            "input_audit.audit_version 不能为空",
        ),
        (
            "manifest_sha256",
            Value::Null,
            "input_audit.manifest_sha256 不能为空",
        ),
        (
            "manifest_sha256",
            json!("tampered"),
            "赛前输入清单 SHA256 与实际清单不一致",
        ),
    ] {
        let mut invalid = input.clone();
        invalid["input_audit"][field] = value;
        assert!(
            matches!(prediction_input_audit_summary(&invalid, "input-sha"), Err(ApplicationError::Model(actual)) if actual == message)
        );
    }
    let mut missing = json!({"input_audit": {"audit_version": "v"}});
    assert!(
        matches!(prediction_input_audit_summary(&missing, "input-sha"), Err(ApplicationError::Model(message)) if message == "input_audit.manifest 不能为空")
    );
    missing["input_audit"]["manifest"] = Value::Null;
    missing["input_audit"]["manifest_sha256"] = json!(sha256_value(&Value::Null).unwrap());
    missing["input_audit"]["audit_version"] = json!(" v ");
    let summary = prediction_input_audit_summary(&missing, "input-sha")
        .unwrap()
        .unwrap();
    assert_eq!(summary.audit_version, " v ");
    assert_eq!(summary.readiness_level, "not_assessed");
    assert_eq!(summary.readiness_score, None);
    assert_eq!(summary.input_sha256, "input-sha");
    for score in [json!("42"), json!(-1), json!(256), json!(42.5), Value::Null] {
        missing["input_audit"]["readiness"] = json!({"score": score, "level": "custom_level"});
        let summary = prediction_input_audit_summary(&missing, "input-sha")
            .unwrap()
            .unwrap();
        assert_eq!(summary.readiness_score, None);
        assert_eq!(summary.readiness_level, "custom_level");
    }
    missing["input_audit"]["readiness"]["score"] = json!(255);
    assert_eq!(
        prediction_input_audit_summary(&missing, "input-sha")
            .unwrap()
            .unwrap()
            .readiness_score,
        Some(255)
    );
}

#[tokio::test]
async fn invalid_audit_stops_execution_before_prediction_and_history_write() {
    use crate::use_cases::prediction::tests::{command_fixture, probe_registry, Probe};
    use std::sync::Arc;
    for persist_run in [true, false] {
        for corrupt_hash in [true, false] {
            let port = Arc::new(Probe::new());
            let mut command = command_fixture();
            let manifest = json!({"match": "A"});
            command.match_input["input_audit"] = json!({"audit_version": PREDICTION_INPUT_AUDIT_VERSION, "manifest": manifest, "manifest_sha256": sha256_value(&manifest).unwrap()});
            let message = if corrupt_hash {
                command.match_input["input_audit"]["manifest"]["match"] = json!("B");
                "赛前输入清单 SHA256 与实际清单不一致"
            } else {
                command.match_input["input_audit"]
                    .as_object_mut()
                    .unwrap()
                    .remove("audit_version");
                "input_audit.audit_version 不能为空"
            };
            let result = super::super::execute_prediction::execute_internal(
                port.as_ref(),
                &probe_registry(&port),
                command,
                persist_run,
            )
            .await;
            assert!(matches!(result, Err(ApplicationError::Model(actual)) if actual == message));
            assert_eq!(port.calls(), ["scope", "route", "model_supports"]);
        }
    }
}
