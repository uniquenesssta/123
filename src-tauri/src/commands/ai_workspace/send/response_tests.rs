use super::response::assistant_message;
use football_research_gateway::PlainTextGatewayExecution;
use serde_json::{json, Value};
use uuid::Uuid;

fn execution(costs: &[Option<f64>]) -> PlainTextGatewayExecution {
    let attempts = costs.iter().map(|cost| json!({
        "attempt_number":1,"model_id":"model","request_fingerprint":"hash","request_payload":{},
        "response_id":null,"provider_request_id":null,"provider_status":200,"status":"success",
        "usage":{"input_tokens":0,"cached_input_tokens":0,"output_tokens":0,"total_tokens":0},
        "latency_ms":1,"search_call_count":0,"estimated_cost_usd":cost,"raw_response":null,
        "error":null,"started_at":"2026-01-01T00:00:00Z","finished_at":"2026-01-01T00:00:00Z"
    })).collect::<Vec<_>>();
    serde_json::from_value(json!({
        "response":{
            "response_id":"","model_id":"model","status":"completed","text":" 原始回答 ",
            "usage":{"input_tokens":9,"cached_input_tokens":3,"output_tokens":4,"total_tokens":13},
            "provider_request_id":"request","raw_response":{"private":"not persisted"}
        },"attempts":attempts
    }))
    .unwrap()
}

#[test]
fn assistant_projection_keeps_usage_ids_raw_text_and_legacy_empty_collections() {
    let result = assistant_message(Uuid::nil(), execution(&[]));
    assert_eq!(result.session_id, Uuid::nil());
    assert_eq!(result.role, "assistant");
    assert_eq!(result.content, " 原始回答 ");
    assert_eq!(result.provider_response_id.as_deref(), Some(""));
    assert_eq!(result.model_id.as_deref(), Some("model"));
    assert_eq!(
        result.structured_payload,
        json!({"mode":"plain_text","legacy":false})
    );
    assert_eq!(result.citations, json!([]));
    assert_eq!(result.attachments, json!([]));
    assert_eq!(
        result.token_usage,
        json!({"input_tokens":9,"cached_input_tokens":3,"output_tokens":4,"total_tokens":13,"estimated_cost_usd":null})
    );
}

#[test]
fn assistant_cost_uses_only_the_last_attempt_including_explicit_none() {
    for (costs, expected) in [
        (vec![Some(1.0), Some(2.5)], json!(2.5)),
        (vec![Some(1.0), None], Value::Null),
    ] {
        let result = assistant_message(Uuid::nil(), execution(&costs));
        assert_eq!(result.token_usage["estimated_cost_usd"], expected);
    }
}
