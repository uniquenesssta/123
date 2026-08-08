use super::shared::audit::{
    build_prediction_input_manifest, prediction_input_audit_summary, sha256_value,
};
use super::shared::routing::{ensure_model_selection_registered, normalize_model_selection};
use crate::{ApplicationError, ApplicationService};
use chrono::Utc;
use football_domain::PREDICTION_INPUT_AUDIT_VERSION;
use serde_json::json;
use uuid::Uuid;

#[test]
fn model_selection_supports_family_and_exact_ids() {
    let p4 = normalize_model_selection("p4").expect("P4 系列必须有效");
    assert_eq!(p4.family, "p4");
    assert!(p4.exact_model_id.is_none());

    let p7 = normalize_model_selection("P7_KNOCKOUT_90").expect("P7 精确模型必须有效");
    assert_eq!(p7.family, "p7");
    assert_eq!(p7.exact_model_id.as_deref(), Some("p7_knockout_90"));
}

#[test]
fn exact_model_must_exist_in_registry() {
    let service = ApplicationService::new();
    let registered = normalize_model_selection("p7_league").expect("内置模型必须有效");
    ensure_model_selection_registered(&service.registry, &registered).expect("已注册模型必须通过");

    let missing = normalize_model_selection("p7_not_registered").expect("格式合法");
    assert!(matches!(
        ensure_model_selection_registered(&service.registry, &missing),
        Err(ApplicationError::ModelNotFound(_))
    ));
}

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
