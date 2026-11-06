use super::parse_history_summary;
use serde_json::{json, Value};

#[test]
fn absent_history_summary_remains_null() {
    let summary = parse_history_summary(Value::Null).unwrap();
    assert!(summary.is_none());
    assert_eq!(serde_json::to_value(summary).unwrap(), Value::Null);
}

#[test]
fn partial_history_summary_does_not_invent_probabilities() {
    let summary = parse_history_summary(json!({"home_win": 0.51})).unwrap();
    let item = summary.as_ref().unwrap();
    assert_eq!(item.home_win, Some(0.51));
    assert_eq!(item.draw, None);
    assert_eq!(item.away_win, None);
    assert_eq!(
        serde_json::to_value(summary).unwrap(),
        json!({"home_win": 0.51})
    );
}

#[test]
fn complete_history_summary_preserves_values_and_rejects_wrong_types() {
    let value = json!({"home_win": 0.51,"draw": 0.25,"away_win": 0.24,"btts": 0.6,"over_2_5": 0.5});
    assert_eq!(
        serde_json::to_value(parse_history_summary(value.clone()).unwrap()).unwrap(),
        value
    );
    for invalid in [
        json!("missing"),
        json!({"home_win": "unknown"}),
        json!([0.5]),
    ] {
        assert!(parse_history_summary(invalid).is_err());
    }
}
