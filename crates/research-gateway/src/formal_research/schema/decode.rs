use crate::{GatewayError, GatewayErrorCategory, ResearchOutput};

pub(crate) fn decode_output_text(output_text: &str) -> Result<ResearchOutput, GatewayError> {
    serde_json::from_str(output_text).map_err(|error| {
        GatewayError::new(
            GatewayErrorCategory::SchemaValidation,
            format!("OpenAI结构化输出无法反序列化：{error}"),
            false,
            "保留原始响应，发布修正后的Prompt或Schema版本后重试",
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    use serde_json::{json, Value};

    fn response_fixture() -> Value {
        json!({"schema_version":"schema-v1","match_key":"match-1","data_cutoff_at":"2026-07-14T10:00:00Z",
            "facts":[{"fact_key":"fact-1","field_key":"home_injuries",
                "subject":{"entity_type":"team","name":"Home","external_id":null},
                "value":{"kind":"string","text":"news","number":null,"integer":null,"boolean":null,"strings":[]},
                "verification_state":"CONFIRMED","source_urls":["https://example.com/news"],
                "published_at":null,"observed_at":null,"effective_at":null,"timezone":null}],
            "missing_fields":[{"field_key":"away_injuries","verification_state":"NOT_FOUND"}]})
    }

    #[test]
    fn typed_decode_rejects_unknown_fields_at_every_annotated_level() {
        for pointer in [
            "",
            "/facts/0",
            "/facts/0/subject",
            "/facts/0/value",
            "/missing_fields/0",
        ] {
            let mut value = response_fixture();
            value
                .pointer_mut(pointer)
                .unwrap()
                .as_object_mut()
                .unwrap()
                .insert("unexpected_field".to_string(), json!(true));
            let error = decode_output_text(&value.to_string()).expect_err("unknown fields");
            assert_eq!(error.category, GatewayErrorCategory::SchemaValidation);
            assert!(error.user_message.contains("unknown field"));
            assert!(!error.recovery.retryable);
            assert_eq!(
                error.recovery.action,
                "保留原始响应，发布修正后的Prompt或Schema版本后重试"
            );
        }
        assert_eq!(
            decode_output_text(&response_fixture().to_string())
                .expect("typed")
                .facts
                .len(),
            1
        );
    }

    #[test]
    fn typed_decode_rejects_missing_fields_wrong_types_and_text_fallbacks() {
        let mut missing = response_fixture();
        missing.as_object_mut().unwrap().remove("schema_version");
        let mut wrong = response_fixture();
        wrong["facts"][0]["source_urls"] = json!("not an array");
        let mut invalid_time = response_fixture();
        invalid_time["data_cutoff_at"] = json!("not a timestamp");
        for text in [
            missing.to_string(),
            wrong.to_string(),
            invalid_time.to_string(),
            "[]".to_string(),
            "ordinary text".to_string(),
            format!("```json\n{}\n```", response_fixture()),
        ] {
            let error = decode_output_text(&text).expect_err("strict decoding");
            assert_eq!(error.category, GatewayErrorCategory::SchemaValidation);
            assert!(error
                .user_message
                .starts_with("OpenAI结构化输出无法反序列化："));
        }
    }
}
