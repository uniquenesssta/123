use super::error::missing_key;
use crate::{GatewayError, GatewayErrorCategory};
use std::fmt;
use zeroize::{Zeroize, Zeroizing};

pub struct ApiKey(String);

impl ApiKey {
    pub fn new(value: String) -> Result<Self, GatewayError> {
        let mut value = Zeroizing::new(value);
        let mut normalized = Zeroizing::new(value.trim().to_string());
        value.zeroize();
        if normalized.is_empty() {
            return Err(missing_key("OpenAI API密钥为空"));
        }
        if normalized.len() > 2_560 || normalized.chars().any(char::is_whitespace) {
            return Err(GatewayError::new(
                GatewayErrorCategory::InvalidConfiguration,
                "OpenAI API密钥格式无效",
                false,
                "粘贴不包含空格、换行或额外说明文字的完整API密钥",
            ));
        }
        Ok(Self(std::mem::take(&mut *normalized)))
    }

    pub(crate) fn expose(&self) -> &str {
        &self.0
    }
}

impl Drop for ApiKey {
    fn drop(&mut self) {
        self.0.zeroize();
    }
}

impl fmt::Debug for ApiKey {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("ApiKey([REDACTED])")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn api_key_debug_is_redacted() {
        let key = ApiKey::new("fixture-credential-value".to_string()).expect("key");
        assert_eq!(format!("{key:?}"), "ApiKey([REDACTED])");
        assert!(!format!("{key:?}").contains("fixture-credential-value"));
    }

    #[test]
    fn api_key_rejects_embedded_whitespace() {
        assert!(ApiKey::new("fixture key".to_string()).is_err());
        assert!(ApiKey::new(" fixture-key\n".to_string()).is_ok());
    }

    #[test]
    fn empty_and_invalid_keys_keep_exact_safe_errors() {
        let empty = ApiKey::new(" \t\n".to_string()).expect_err("empty");
        assert_eq!(empty, missing_key("OpenAI API密钥为空"));
        let fixture = "fixture secret";
        let error = ApiKey::new(fixture.to_string()).expect_err("invalid");
        assert_eq!(
            error,
            GatewayError::new(
                GatewayErrorCategory::InvalidConfiguration,
                "OpenAI API密钥格式无效",
                false,
                "粘贴不包含空格、换行或额外说明文字的完整API密钥",
            )
        );
        for rendered in [
            format!("{error:?}"),
            error.to_string(),
            serde_json::to_string(&error).expect("serialize"),
        ] {
            assert!(!rendered.contains(fixture));
        }
    }

    #[test]
    fn byte_limit_trimming_and_unicode_policy_remain_compatible() {
        let key = ApiKey::new(format!("  {}\n", "k".repeat(2_560))).expect("limit");
        assert_eq!(key.expose(), "k".repeat(2_560));
        assert!(ApiKey::new("k".repeat(2_561)).is_err());
        assert!(ApiKey::new("密".repeat(853)).is_ok());
        assert!(ApiKey::new("密".repeat(854)).is_err());
        assert!(ApiKey::new("fixture\u{2003}key".to_string()).is_err());
    }
}
