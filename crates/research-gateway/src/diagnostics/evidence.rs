use crate::{GatewayError, GatewayErrorCategory};
use serde_json::Value;
use sha2::{Digest, Sha256};

pub(crate) fn response_id_from_body(body: &Value) -> Option<String> {
    body.get("id")
        .and_then(Value::as_str)
        .map(ToString::to_string)
}

pub(crate) fn fingerprint(value: &Value) -> Result<String, GatewayError> {
    let bytes = serde_json::to_vec(value).map_err(|error| {
        GatewayError::new(
            GatewayErrorCategory::InvalidConfiguration,
            format!("OpenAI请求无法计算指纹：{error}"),
            false,
            "修正请求载荷后重试",
        )
    })?;
    Ok(hex::encode(Sha256::digest(bytes)))
}
