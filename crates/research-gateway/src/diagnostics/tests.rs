use super::{
    elapsed_millis, fingerprint, response_id_from_body, GatewayAttempt, GatewayAttemptSink,
    NoopAttemptSink,
};
use crate::{GatewayError, GatewayErrorCategory, GatewayUsage};
use chrono::{TimeZone, Utc};
use serde_json::json;
use std::time::{Duration, Instant};

fn attempt() -> GatewayAttempt {
    GatewayAttempt {
        attempt_number: 8,
        model_id: "fixture-model".to_string(),
        request_fingerprint: "fixture-fingerprint".to_string(),
        request_payload: json!({"input": "private request evidence"}),
        response_id: Some("resp_fixture".to_string()),
        provider_request_id: Some("req_fixture".to_string()),
        provider_status: Some(429),
        status: "failed".to_string(),
        usage: GatewayUsage {
            input_tokens: 12,
            cached_input_tokens: 3,
            output_tokens: 4,
            total_tokens: 16,
        },
        latency_ms: 1500,
        search_call_count: 2,
        estimated_cost_usd: Some(0.003),
        raw_response: Some(json!({"error": {"code": "rate_limit"}})),
        error: Some(
            GatewayError::new(
                GatewayErrorCategory::RateLimit,
                "fixture rate limit",
                true,
                "fixture retry",
            )
            .with_provider(Some(429), Some("rate_limit".to_string())),
        ),
        started_at: Utc.with_ymd_and_hms(2026, 10, 12, 0, 0, 0).unwrap(),
        finished_at: Utc.with_ymd_and_hms(2026, 10, 12, 0, 0, 2).unwrap(),
    }
}

#[test]
fn request_fingerprint_keeps_utf8_json_bytes_and_lowercase_sha256() {
    assert_eq!(
        fingerprint(&json!("雪")).unwrap(),
        "b5e24bd48f8cf5d82e8806ee3454cdf6541d1a4205ed8ff695b65db36856990f"
    );
}

#[test]
fn request_fingerprint_preserves_array_order_and_json_value_types() {
    assert_eq!(
        fingerprint(&json!(["雪", 1, false, null])).unwrap(),
        "0a388e03d397e8fec24d94eca8fa0cc436c5b8c2904366fba29cdfec9e6e23b8"
    );
    assert_eq!(
        fingerprint(&json!(["雪", false, 1, null])).unwrap(),
        "1b605cafa9cd4908571a39cb0b6fbc65ec3d2d095aef8678b0748b6eed4db64b"
    );
    assert_ne!(
        fingerprint(&json!(1)).unwrap(),
        fingerprint(&json!("1")).unwrap()
    );
}

#[test]
fn response_evidence_id_keeps_original_strings_and_rejects_other_types() {
    for id in ["resp_fixture", "", "  resp_fixture  "] {
        assert_eq!(
            response_id_from_body(&json!({"id": id})),
            Some(id.to_string())
        );
    }
    for body in [
        json!({}),
        json!({"id": null}),
        json!({"id": 7}),
        json!({"id": true}),
        json!({"id": []}),
    ] {
        assert_eq!(response_id_from_body(&body), None);
    }
}

#[test]
fn attempt_serialization_preserves_all_public_evidence_and_optional_nulls() {
    let original = attempt();
    let expected = json!({
        "attempt_number": 8, "model_id": "fixture-model", "request_fingerprint": "fixture-fingerprint",
        "request_payload": {"input": "private request evidence"}, "response_id": "resp_fixture",
        "provider_request_id": "req_fixture", "provider_status": 429, "status": "failed",
        "usage": {"input_tokens": 12, "cached_input_tokens": 3, "output_tokens": 4, "total_tokens": 16},
        "latency_ms": 1500, "search_call_count": 2, "estimated_cost_usd": 0.003,
        "raw_response": {"error": {"code": "rate_limit"}},
        "error": {"category": "rate_limit", "user_message": "fixture rate limit", "recovery": {"retryable": true, "action": "fixture retry"}, "provider_status": 429, "provider_code": "rate_limit"},
        "started_at": "2026-10-12T00:00:00Z", "finished_at": "2026-10-12T00:00:02Z"
    });
    assert_eq!(serde_json::to_value(&original).unwrap(), expected);
    assert_eq!(
        serde_json::from_value::<GatewayAttempt>(expected).unwrap(),
        original
    );

    let mut empty = original;
    empty.response_id = None;
    empty.provider_request_id = None;
    empty.provider_status = None;
    empty.estimated_cost_usd = None;
    empty.raw_response = None;
    empty.error = None;
    let encoded = serde_json::to_value(&empty).unwrap();
    for field in [
        "response_id",
        "provider_request_id",
        "provider_status",
        "estimated_cost_usd",
        "raw_response",
        "error",
    ] {
        assert!(encoded.as_object().unwrap().contains_key(field));
        assert!(encoded[field].is_null());
    }
    assert_eq!(
        serde_json::from_value::<GatewayAttempt>(encoded).unwrap(),
        empty
    );
}

#[tokio::test]
async fn default_attempt_sink_accepts_evidence_through_the_original_dyn_interface() {
    let evidence = attempt();
    let before = serde_json::to_value(&evidence).unwrap();
    let sink: &dyn GatewayAttemptSink = &NoopAttemptSink;
    assert_eq!(sink.record(&evidence).await, Ok(()));
    assert_eq!(serde_json::to_value(&evidence).unwrap(), before);
}

#[test]
fn latency_reports_elapsed_milliseconds_without_waiting_or_rounding_to_seconds() {
    let started = Instant::now()
        .checked_sub(Duration::from_millis(1500))
        .unwrap();
    assert!(elapsed_millis(started) >= 1500);
}
