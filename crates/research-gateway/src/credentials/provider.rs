use super::{error::missing_key, windows::load_windows_credential, ApiKey};
use crate::{CredentialConfig, CredentialMode, GatewayError, GatewayErrorCategory};
use async_trait::async_trait;

#[async_trait]
pub trait ApiKeyProvider: Send + Sync {
    async fn load(&self, config: &CredentialConfig) -> Result<ApiKey, GatewayError>;
}

#[derive(Debug, Default, Clone, Copy)]
pub struct DefaultApiKeyProvider;

#[async_trait]
impl ApiKeyProvider for DefaultApiKeyProvider {
    async fn load(&self, config: &CredentialConfig) -> Result<ApiKey, GatewayError> {
        match config.mode {
            CredentialMode::WindowsCredentialManager => {
                load_windows_credential(&config.credential_target)
            }
            CredentialMode::ServerEnvironment => {
                if config.deployment_mode != "server" {
                    return Err(GatewayError::new(
                        GatewayErrorCategory::InvalidConfiguration,
                        "桌面部署禁止从环境变量读取OpenAI API密钥",
                        false,
                        "改用Windows凭据管理器，或将部署模式明确设置为server",
                    ));
                }
                let value = std::env::var(&config.environment_variable).map_err(|_| {
                    missing_key(format!(
                        "服务器环境变量{}尚未配置",
                        config.environment_variable
                    ))
                })?;
                ApiKey::new(value)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn environment_config(deployment_mode: &str) -> CredentialConfig {
        CredentialConfig {
            mode: CredentialMode::ServerEnvironment,
            deployment_mode: deployment_mode.to_string(),
            environment_variable: "FOOTBALL_TEST_R902_MISSING_87950FB11A624C93".to_string(),
            credential_target: "football-match-model-platform/openai/fixture".to_string(),
        }
    }

    #[tokio::test]
    async fn desktop_environment_is_rejected_before_reading_missing_variable() {
        for deployment in ["local_desktop", "SERVER", "server "] {
            let error = DefaultApiKeyProvider
                .load(&environment_config(deployment))
                .await
                .expect_err("blocked");
            assert_eq!(
                error,
                GatewayError::new(
                    GatewayErrorCategory::InvalidConfiguration,
                    "桌面部署禁止从环境变量读取OpenAI API密钥",
                    false,
                    "改用Windows凭据管理器，或将部署模式明确设置为server",
                )
            );
        }
    }

    #[tokio::test]
    async fn explicit_server_missing_variable_keeps_original_recovery() {
        let config = environment_config("server");
        assert!(std::env::var_os(&config.environment_variable).is_none());
        let error = DefaultApiKeyProvider
            .load(&config)
            .await
            .expect_err("missing");
        assert_eq!(
            error,
            missing_key(format!(
                "服务器环境变量{}尚未配置",
                config.environment_variable
            ))
        );
        assert_eq!(
            format!("{:?}", DefaultApiKeyProvider),
            "DefaultApiKeyProvider"
        );
    }
}
