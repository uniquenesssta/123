use super::policy::{
    ability_values, ensure_pending, json_string, json_uuid, suggestion_severity,
    validate_import_preview, AbilitySuggestionValues,
};
use crate::PersistenceError;
use football_domain::AiAnalysisResponsePreview;
use serde_json::json;
use uuid::Uuid;

fn preview() -> AiAnalysisResponsePreview {
    serde_json::from_value(json!({"manifest": {"format_version": "football.ai-analysis-response.v1", "response_id": Uuid::from_u128(1),
        "created_at": "2026-10-13T00:00:00.123456789Z", "source_package_id": null, "content_sha256": "original"}, "suggestions": [], "blocking_errors": [], "warnings": []})).unwrap()
}

#[test]
fn blocking_preview_errors_keep_original_order_and_separator() {
    let mut input = preview();
    input.blocking_errors = vec!["first".into(), " second ".into(), String::new()];
    assert!(
        matches!(validate_import_preview(&input).unwrap_err(), PersistenceError::InvalidState(message) if message == "AI 回包存在阻断错误：first； second ；")
    );
}

#[test]
fn warnings_and_empty_suggestions_do_not_become_blocking_errors() {
    let mut input = preview();
    input.warnings.push("empty suggestions".into());
    validate_import_preview(&input).unwrap();
}

#[test]
fn severity_defaults_only_for_blank_and_otherwise_keeps_original_bytes() {
    for value in ["", " ", "\t\n"] {
        assert_eq!(suggestion_severity(value), "info");
    }
    for value in ["critical", " warning ", "Unknown", "信息"] {
        assert_eq!(suggestion_severity(value), value);
    }
}

#[test]
fn manual_decision_requires_exact_pending_status() {
    ensure_pending("pending").unwrap();
    for status in ["accepted", "rejected", " pending", "Pending", "", "unknown"] {
        assert!(
            matches!(ensure_pending(status).unwrap_err(), PersistenceError::InvalidState(message) if message == "该 AI 建议已处理")
        );
    }
}

#[test]
fn payload_string_values_remain_untrimmed_and_wrong_types_keep_error() {
    assert_eq!(
        json_string(&json!({"dimension_code": " speed "}), "dimension_code").unwrap(),
        " speed "
    );
    for payload in [
        json!({}),
        json!({"dimension_code": null}),
        json!({"dimension_code": 3}),
        json!({"dimension_code": true}),
    ] {
        assert!(
            matches!(json_string(&payload, "dimension_code").unwrap_err(), PersistenceError::InvalidState(message) if message == "AI 建议缺少 dimension_code")
        );
    }
}

#[test]
fn payload_uuid_preserves_valid_nil_and_original_missing_or_invalid_errors() {
    for id in [Uuid::nil(), Uuid::from_u128(99)] {
        assert_eq!(
            json_uuid(&json!({"player_id": id}), "player_id").unwrap(),
            id
        );
    }
    assert!(
        matches!(json_uuid(&json!({}), "player_id").unwrap_err(), PersistenceError::InvalidState(message) if message == "AI 建议缺少 player_id")
    );
    assert!(
        matches!(json_uuid(&json!({"player_id": "bad"}), "player_id").unwrap_err(), PersistenceError::InvalidState(message) if message.starts_with("AI 建议 player_id 无效："))
    );
}

#[test]
fn ability_values_keep_original_clamps_defaults_and_integer_type_policy() {
    for (payload, expected) in [
        (
            json!({"proposed_value": 87.5, "confidence": 0.25, "sample_size": 42}),
            AbilitySuggestionValues {
                proposed_value: 87.5,
                confidence: 0.25,
                sample_size: 42,
            },
        ),
        (
            json!({"proposed_value": -20, "confidence": -1, "sample_size": -2}),
            AbilitySuggestionValues {
                proposed_value: 0.0,
                confidence: 0.0,
                sample_size: 0,
            },
        ),
        (
            json!({"proposed_value": 150, "confidence": 2, "sample_size": i64::MAX}),
            AbilitySuggestionValues {
                proposed_value: 100.0,
                confidence: 1.0,
                sample_size: i32::MAX,
            },
        ),
        (
            json!({"proposed_value": 42}),
            AbilitySuggestionValues {
                proposed_value: 42.0,
                confidence: 0.5,
                sample_size: 0,
            },
        ),
        (
            json!({"proposed_value": 42, "confidence": "0.9", "sample_size": 1.5}),
            AbilitySuggestionValues {
                proposed_value: 42.0,
                confidence: 0.5,
                sample_size: 0,
            },
        ),
    ] {
        assert_eq!(ability_values(&payload).unwrap(), expected);
    }
}

#[test]
fn ability_proposed_value_must_be_numeric_before_optional_defaults() {
    for payload in [
        json!({}),
        json!({"proposed_value": null}),
        json!({"proposed_value": "90"}),
        json!({"proposed_value": true}),
    ] {
        assert!(
            matches!(ability_values(&payload).unwrap_err(), PersistenceError::InvalidState(message) if message == "AI 建议缺少数值 proposed_value")
        );
    }
}
