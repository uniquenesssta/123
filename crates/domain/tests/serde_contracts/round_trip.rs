use football_domain::{LineupDraft, MatchContext, RulePackageDraft};
use serde_json::{json, Value};

fn fixture(name: &str) -> &'static str {
    match name {
        "match_context" => include_str!("fixtures/match_context.json"),
        "rule_package" => include_str!("fixtures/rule_package.json"),
        "lineup" => include_str!("fixtures/lineup.json"),
        _ => panic!("unknown fixture"),
    }
}

#[test]
fn historical_match_context_json_remains_readable() {
    let context: MatchContext = serde_json::from_str(fixture("match_context"))
        .expect("legacy match context should deserialize");
    assert_eq!(context.match_key, "legacy-match");
    assert_eq!(context.competition_kind.as_str(), "custom");
    assert!(context.competition_id.is_none());
    assert_eq!(context.metadata, Value::Null);

    let serialized = serde_json::to_value(context).expect("match context should serialize");
    assert_eq!(serialized["competition_kind"], json!("custom"));
    assert_eq!(serialized["metadata"], Value::Null);
}

#[test]
fn rule_package_defaults_and_nested_contract_round_trip() {
    let package: RulePackageDraft = serde_json::from_str(fixture("rule_package"))
        .expect("legacy rule package should deserialize");
    assert_eq!(package.format_version, "football.rule-package.v1");
    assert_eq!(package.competition_profile.normal_time_minutes, 90);
    assert_eq!(package.routing.priority, 0);
    assert!(package.routing.supported_snapshot_types.is_empty());
    assert_eq!(package.feature_requirements, Value::Null);

    let serialized = serde_json::to_value(package).expect("rule package should serialize");
    assert_eq!(serialized["competition_profile"]["competition_kind"], json!("league"));
    assert_eq!(serialized["format_version"], json!("football.rule-package.v1"));
}

#[test]
fn lineup_legacy_json_keeps_default_snapshot_and_member_fields() {
    let lineup: LineupDraft = serde_json::from_str(fixture("lineup"))
        .expect("legacy lineup should deserialize");
    assert_eq!(lineup.snapshot_type, "T-1h");
    assert_eq!(lineup.players.len(), 1);
    let player = &lineup.players[0];
    assert!(player.position_code.is_none());
    assert!(player.role_code.is_none());
    assert!(!player.membership_override);
    assert!(player.source_urls.is_empty());

    let serialized = serde_json::to_value(lineup).expect("lineup should serialize");
    assert_eq!(serialized["snapshot_type"], json!("T-1h"));
    assert_eq!(serialized["players"][0]["sequence_no"], json!(0));
}
