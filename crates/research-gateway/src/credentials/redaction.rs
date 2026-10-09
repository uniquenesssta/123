use serde_json::{json, Map, Value};

pub(crate) const API_KEY_PLACEHOLDER: &str = "YOUR_API_KEY";

pub(crate) fn sanitized_api_example(
    endpoint_url: &str,
    body: &Value,
    api_key: Option<&str>,
) -> String {
    canonical_curl(endpoint_url, &sanitize_body(body, api_key))
}

fn redact_known_key(value: &str, api_key: Option<&str>) -> String {
    match api_key.filter(|key| !key.is_empty()) {
        Some(key) => value.replace(key, API_KEY_PLACEHOLDER),
        None => value.to_string(),
    }
}

fn sanitize_body(body: &Value, api_key: Option<&str>) -> Value {
    match body {
        Value::Object(object) => {
            let sanitized = object
                .iter()
                .map(|(key, value)| {
                    (
                        redact_known_key(key, api_key),
                        sanitize_body(value, api_key),
                    )
                })
                .collect::<Map<String, Value>>();
            Value::Object(sanitized)
        }
        Value::Array(values) => Value::Array(
            values
                .iter()
                .map(|value| sanitize_body(value, api_key))
                .collect(),
        ),
        Value::String(value) if looks_like_secret(value) => {
            Value::String(API_KEY_PLACEHOLDER.to_string())
        }
        Value::String(value) => Value::String(redact_known_key(value, api_key)),
        _ => body.clone(),
    }
}

fn canonical_curl(endpoint_url: &str, body: &Value) -> String {
    let pretty = serde_json::to_string_pretty(body).unwrap_or_else(|_| json!({}).to_string());
    format!(
        "curl {endpoint_url} \\\n  -H \"Content-Type: application/json\" \\\n  -H \"Authorization: Bearer {API_KEY_PLACEHOLDER}\" \\\n  -d '{pretty}'"
    )
}

pub(crate) fn is_placeholder_key(value: &str) -> bool {
    let normalized = value.trim().to_ascii_uppercase();
    normalized.is_empty()
        || normalized.contains("YOUR_API_KEY")
        || normalized.contains("API_KEY_HERE")
        || normalized.contains("REPLACE_ME")
        || normalized.starts_with('<')
        || normalized.starts_with("${")
        || normalized == "XXX"
        || normalized == "TOKEN"
}

fn looks_like_secret(value: &str) -> bool {
    let trimmed = value.trim();
    !is_placeholder_key(trimmed)
        && trimmed.len() >= 20
        && !trimmed.chars().any(char::is_whitespace)
        && (trimmed.starts_with("sk-")
            || trimmed.starts_with("sk_")
            || trimmed.starts_with("Bearer "))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nested_body_and_object_keys_never_retain_the_extracted_bearer() {
        let key = "fixture-provider-key";
        let mut body = json!({"input": ["hello", {"embedded": key}], "model": "fixture-model", "limit": 12, "enabled": true, "empty": null});
        body["input"][1].as_object_mut().expect("object").insert(
            key.to_string(),
            Value::String(format!("prefix {key} suffix")),
        );
        let sanitized = sanitize_body(&body, Some(key));
        assert_eq!(
            sanitized,
            json!({"input": ["hello", {"YOUR_API_KEY": "prefix YOUR_API_KEY suffix", "embedded": "YOUR_API_KEY"}], "model": "fixture-model", "limit": 12, "enabled": true, "empty": null})
        );
        let template =
            sanitized_api_example("https://fixture.invalid/v1/responses", &body, Some(key));
        assert!(!template.contains(key));
        assert!(template.contains("Authorization: Bearer YOUR_API_KEY"));
        assert_eq!(body["input"][1]["embedded"], key);
    }

    #[test]
    fn existing_secret_heuristic_and_placeholder_policy_remain_compatible() {
        let body = json!({"input": ["sk-test-fixture-secret-123456", "sk_fixture_secret_1234567890", "YOUR_API_KEY", "${TOKEN}", "ordinary text"], "count": 2});
        assert_eq!(
            sanitize_body(&body, None),
            json!({"input": ["YOUR_API_KEY", "YOUR_API_KEY", "YOUR_API_KEY", "${TOKEN}", "ordinary text"], "count": 2})
        );
        for placeholder in [
            "",
            "your_api_key",
            "API_KEY_HERE",
            "replace_me",
            "<key>",
            "${key}",
            "XXX",
            "TOKEN",
        ] {
            assert!(is_placeholder_key(placeholder));
        }
        assert!(!is_placeholder_key("fixture-provider-key"));
    }

    #[test]
    fn unmodified_values_and_empty_optional_key_preserve_canonical_template() {
        let body =
            json!({"model":"fixture-model", "input":"hello", "count": 1, "array": [false, null]});
        assert_eq!(sanitize_body(&body, None), body);
        assert_eq!(sanitize_body(&body, Some("")), body);
        assert_eq!(
            sanitized_api_example("https://fixture.invalid/v1/responses", &body, None),
            canonical_curl("https://fixture.invalid/v1/responses", &body)
        );
    }
}
