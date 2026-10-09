use super::{
    windows::{delete_windows_credential, windows_credential_exists, write_windows_credential},
    ApiKey,
};
use crate::{GatewayError, GatewayErrorCategory};
use zeroize::Zeroizing;

pub fn save_windows_api_key(target: &str, value: String) -> Result<(), GatewayError> {
    let mut value = Zeroizing::new(value);
    validate_credential_target(target)?;
    let key = ApiKey::new(std::mem::take(&mut *value))?;
    write_windows_credential(target, &key)
}

pub fn delete_windows_api_key(target: &str) -> Result<(), GatewayError> {
    validate_credential_target(target)?;
    delete_windows_credential(target)
}

pub fn windows_api_key_exists(target: &str) -> Result<bool, GatewayError> {
    validate_credential_target(target)?;
    windows_credential_exists(target)
}

fn validate_credential_target(target: &str) -> Result<(), GatewayError> {
    let valid = !target.trim().is_empty()
        && target.len() <= 240
        && target
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'/' | b'.'));
    if valid {
        Ok(())
    } else {
        Err(GatewayError::new(
            GatewayErrorCategory::InvalidConfiguration,
            "Windows凭据目标格式无效",
            false,
            "使用应用生成的OpenAI配置档案标识，不要手工拼接凭据目标",
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn credential_target_rejects_path_injection() {
        assert!(
            validate_credential_target("football-match-model-platform/openai/profile-1").is_ok()
        );
        assert!(validate_credential_target("../openai?secret").is_err());
    }

    #[test]
    fn target_limits_and_original_character_policy_remain_compatible() {
        for valid in ["A-z_9/..", "../openai", &"a".repeat(240)] {
            assert!(validate_credential_target(valid).is_ok());
        }
        for invalid in [
            "",
            " ",
            "profile:key",
            "profile\\key",
            "profile?key",
            "profile\nkey",
            "配置",
            &"a".repeat(241),
        ] {
            let error = validate_credential_target(invalid).expect_err("invalid target");
            assert_eq!(
                error,
                GatewayError::new(
                    GatewayErrorCategory::InvalidConfiguration,
                    "Windows凭据目标格式无效",
                    false,
                    "使用应用生成的OpenAI配置档案标识，不要手工拼接凭据目标",
                )
            );
        }
    }

    #[test]
    fn invalid_targets_stop_all_public_operations_before_native_io() {
        let expected = validate_credential_target("fixture?target").expect_err("target");
        assert_eq!(
            save_windows_api_key("fixture?target", " ".to_string()).expect_err("save"),
            expected
        );
        assert_eq!(
            delete_windows_api_key("fixture?target").expect_err("delete"),
            expected
        );
        assert_eq!(
            windows_api_key_exists("fixture?target").expect_err("exists"),
            expected
        );
        let error = save_windows_api_key("fixture-valid-target", "fixture bad key".to_string())
            .expect_err("key");
        assert_eq!(error.user_message, "OpenAI API密钥格式无效");
        assert!(!serde_json::to_string(&error)
            .expect("serialize")
            .contains("fixture bad key"));
    }
}
