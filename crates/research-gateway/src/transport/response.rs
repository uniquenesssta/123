use super::TransportResponse;
use crate::{GatewayError, GatewayErrorCategory};
use serde_json::Value;

pub(super) fn decode_response(
    status: u16,
    provider_request_id: Option<String>,
    bytes: &[u8],
) -> Result<TransportResponse, GatewayError> {
    let body = if bytes.is_empty() {
        Value::Null
    } else {
        serde_json::from_slice(bytes).map_err(|error| {
            GatewayError::new(
                GatewayErrorCategory::SchemaValidation,
                format!("兼容 API返回了无法解析的JSON：{error}"),
                true,
                "保留HTTP状态和请求ID后重试；持续发生时检查API端点配置",
            )
            .with_provider(Some(status), None)
        })?
    };
    Ok(TransportResponse {
        status,
        provider_request_id,
        body,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn transport_preserves_any_json_and_http_status_without_protocol_policy() {
        for status in [200, 400, 429, 500] {
            for body in [
                json!({"error": {"code": "fixture"}}),
                json!([1, "text"]),
                json!(42),
                json!(null),
            ] {
                let bytes = serde_json::to_vec(&body).unwrap();
                let response = decode_response(status, Some("req_fixture".into()), &bytes).unwrap();
                assert_eq!(response.status, status);
                assert_eq!(response.provider_request_id.as_deref(), Some("req_fixture"));
                assert_eq!(response.body, body);
            }
        }
    }

    #[test]
    fn empty_response_is_null_and_preserves_original_metadata() {
        let response = decode_response(204, None, &[]).unwrap();
        assert_eq!(response.status, 204);
        assert_eq!(response.provider_request_id, None);
        assert_eq!(response.body, Value::Null);
        let response = decode_response(503, Some("req_empty".into()), &[]).unwrap();
        assert_eq!(response.status, 503);
        assert_eq!(response.provider_request_id.as_deref(), Some("req_empty"));
        assert_eq!(response.body, Value::Null);
    }

    #[test]
    fn malformed_json_retains_original_error_category_message_and_status() {
        for bytes in [
            b" \n".as_slice(),
            b"{".as_slice(),
            &[0xff],
            b"null\0".as_slice(),
        ] {
            let parse_error = serde_json::from_slice::<Value>(bytes).unwrap_err();
            let expected = GatewayError::new(
                GatewayErrorCategory::SchemaValidation,
                format!("兼容 API返回了无法解析的JSON：{parse_error}"),
                true,
                "保留HTTP状态和请求ID后重试；持续发生时检查API端点配置",
            )
            .with_provider(Some(502), None);
            assert_eq!(
                decode_response(502, Some("req_invalid".into()), bytes).unwrap_err(),
                expected
            );
        }
    }
}
