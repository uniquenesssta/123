use crate::TokenLimitField;
use serde_json::{json, Value};

pub(crate) fn apply_token_limit(body: &mut Value, field: TokenLimitField, value: u32) {
    if let Value::Object(object) = body {
        object.remove("max_output_tokens");
        object.remove("max_tokens");
        object.insert(field.as_str().to_string(), json!(value));
    }
}
