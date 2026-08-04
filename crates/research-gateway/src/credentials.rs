use crate::{CredentialConfig, CredentialMode, GatewayError, GatewayErrorCategory};
use async_trait::async_trait;
use std::fmt;
use zeroize::Zeroize;

pub struct ApiKey(String);

impl ApiKey {
    pub fn new(mut value: String) -> Result<Self, GatewayError> {
        let normalized = value.trim().to_string();
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
        Ok(Self(normalized))
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

#[async_trait]
pub trait ApiKeyProvider: Send + Sync {
    async fn load(&self, config: &CredentialConfig) -> Result<ApiKey, GatewayError>;
}

pub fn save_windows_api_key(target: &str, value: String) -> Result<(), GatewayError> {
    validate_credential_target(target)?;
    let key = ApiKey::new(value)?;
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

#[cfg(windows)]
fn load_windows_credential(target: &str) -> Result<ApiKey, GatewayError> {
    use std::ptr;
    use windows_sys::Win32::Security::Credentials::{
        CredFree, CredReadW, CREDENTIALW, CRED_TYPE_GENERIC,
    };

    let mut target_wide: Vec<u16> = target.encode_utf16().chain(std::iter::once(0)).collect();
    let mut credential: *mut CREDENTIALW = ptr::null_mut();
    let success = unsafe { CredReadW(target_wide.as_ptr(), CRED_TYPE_GENERIC, 0, &mut credential) };
    target_wide.zeroize();
    if success == 0 || credential.is_null() {
        return Err(missing_key(format!(
            "Windows凭据管理器中未找到目标：{target}"
        )));
    }

    let result = unsafe {
        let credential_ref = &*credential;
        let value =
            if credential_ref.CredentialBlob.is_null() || credential_ref.CredentialBlobSize == 0 {
                Err(missing_key("Windows凭据管理器中的OpenAI密钥为空"))
            } else {
                let blob = std::slice::from_raw_parts(
                    credential_ref.CredentialBlob,
                    credential_ref.CredentialBlobSize as usize,
                );
                let mut bytes = blob.to_vec();
                let value = decode_credential_blob(&bytes);
                bytes.zeroize();
                value
            };
        CredFree(credential.cast());
        value
    }?;
    ApiKey::new(result)
}

#[cfg(windows)]
fn write_windows_credential(target: &str, api_key: &ApiKey) -> Result<(), GatewayError> {
    use windows_sys::Win32::Security::Credentials::{
        CredWriteW, CREDENTIALW, CRED_PERSIST_LOCAL_MACHINE, CRED_TYPE_GENERIC,
    };

    let mut target_wide: Vec<u16> = target.encode_utf16().chain(std::iter::once(0)).collect();
    let mut username_wide: Vec<u16> = "openai".encode_utf16().chain(std::iter::once(0)).collect();
    let mut blob = api_key.expose().as_bytes().to_vec();
    if blob.len() > 2_560 {
        blob.zeroize();
        target_wide.zeroize();
        username_wide.zeroize();
        return Err(GatewayError::new(
            GatewayErrorCategory::InvalidConfiguration,
            "OpenAI API密钥长度超过Windows凭据管理器限制",
            false,
            "检查密钥内容后重新保存",
        ));
    }
    let mut credential: CREDENTIALW = unsafe { std::mem::zeroed() };
    credential.Type = CRED_TYPE_GENERIC;
    credential.TargetName = target_wide.as_mut_ptr();
    credential.CredentialBlobSize = blob.len() as u32;
    credential.CredentialBlob = blob.as_mut_ptr();
    credential.Persist = CRED_PERSIST_LOCAL_MACHINE;
    credential.UserName = username_wide.as_mut_ptr();
    let success = unsafe { CredWriteW(&credential, 0) };
    blob.zeroize();
    target_wide.zeroize();
    username_wide.zeroize();
    if success == 0 {
        Err(GatewayError::new(
            GatewayErrorCategory::Persistence,
            format!(
                "无法写入Windows凭据管理器：{}",
                std::io::Error::last_os_error()
            ),
            true,
            "确认当前Windows用户允许保存普通凭据后重试",
        ))
    } else {
        Ok(())
    }
}

#[cfg(not(windows))]
fn write_windows_credential(_target: &str, _api_key: &ApiKey) -> Result<(), GatewayError> {
    Err(GatewayError::new(
        GatewayErrorCategory::MissingCredential,
        "当前系统不是Windows，无法保存Windows凭据",
        false,
        "请在Windows桌面客户端中保存OpenAI API密钥",
    ))
}

#[cfg(windows)]
fn delete_windows_credential(target: &str) -> Result<(), GatewayError> {
    use windows_sys::Win32::Security::Credentials::{CredDeleteW, CRED_TYPE_GENERIC};
    if !windows_credential_exists(target)? {
        return Ok(());
    }
    let mut target_wide: Vec<u16> = target.encode_utf16().chain(std::iter::once(0)).collect();
    let success = unsafe { CredDeleteW(target_wide.as_ptr(), CRED_TYPE_GENERIC, 0) };
    target_wide.zeroize();
    if success == 0 {
        Err(GatewayError::new(
            GatewayErrorCategory::Persistence,
            format!("无法删除Windows凭据：{}", std::io::Error::last_os_error()),
            true,
            "关闭可能占用凭据的程序后重试",
        ))
    } else {
        Ok(())
    }
}

#[cfg(not(windows))]
fn delete_windows_credential(_target: &str) -> Result<(), GatewayError> {
    Err(GatewayError::new(
        GatewayErrorCategory::MissingCredential,
        "当前系统不是Windows，无法删除Windows凭据",
        false,
        "请在Windows桌面客户端中管理OpenAI API密钥",
    ))
}

#[cfg(windows)]
fn windows_credential_exists(target: &str) -> Result<bool, GatewayError> {
    use std::ptr;
    use windows_sys::Win32::Security::Credentials::{
        CredFree, CredReadW, CREDENTIALW, CRED_TYPE_GENERIC,
    };
    let mut target_wide: Vec<u16> = target.encode_utf16().chain(std::iter::once(0)).collect();
    let mut credential: *mut CREDENTIALW = ptr::null_mut();
    let success = unsafe { CredReadW(target_wide.as_ptr(), CRED_TYPE_GENERIC, 0, &mut credential) };
    target_wide.zeroize();
    if success == 0 {
        let error = std::io::Error::last_os_error();
        if error.raw_os_error() == Some(1_168) {
            return Ok(false);
        }
        return Err(GatewayError::new(
            GatewayErrorCategory::Persistence,
            format!("无法读取Windows凭据状态：{error}"),
            true,
            "确认当前Windows用户可以访问凭据管理器后重试",
        ));
    }
    if credential.is_null() {
        return Err(GatewayError::new(
            GatewayErrorCategory::Persistence,
            "Windows凭据管理器返回了空凭据指针",
            true,
            "重新打开客户端后重试",
        ));
    }
    unsafe { CredFree(credential.cast()) };
    Ok(true)
}

#[cfg(not(windows))]
fn windows_credential_exists(_target: &str) -> Result<bool, GatewayError> {
    Ok(false)
}

#[cfg(any(windows, test))]
fn decode_credential_blob(bytes: &[u8]) -> Result<String, GatewayError> {
    if looks_like_utf16le(bytes) {
        let units: Vec<u16> = bytes
            .chunks_exact(2)
            .map(|chunk| u16::from_le_bytes([chunk[0], chunk[1]]))
            .take_while(|unit| *unit != 0)
            .collect();
        if let Ok(value) = String::from_utf16(&units) {
            let trimmed = value.trim();
            if !trimmed.is_empty() {
                return Ok(trimmed.to_string());
            }
        }
    }
    if let Ok(value) = std::str::from_utf8(bytes) {
        let trimmed = value.trim_matches(char::from(0)).trim();
        if !trimmed.is_empty() && !trimmed.contains(char::from(0)) {
            return Ok(trimmed.to_string());
        }
    }
    if bytes.len() % 2 == 0 {
        let units: Vec<u16> = bytes
            .chunks_exact(2)
            .map(|chunk| u16::from_le_bytes([chunk[0], chunk[1]]))
            .take_while(|unit| *unit != 0)
            .collect();
        if let Ok(value) = String::from_utf16(&units) {
            let trimmed = value.trim();
            if !trimmed.is_empty() {
                return Ok(trimmed.to_string());
            }
        }
    }
    Err(missing_key(
        "Windows凭据管理器中的OpenAI密钥不是有效UTF-8或UTF-16LE文本",
    ))
}

#[cfg(any(windows, test))]
fn looks_like_utf16le(bytes: &[u8]) -> bool {
    bytes.len() >= 2
        && bytes.len() % 2 == 0
        && bytes
            .chunks_exact(2)
            .take(8)
            .any(|chunk| chunk[1] == 0 && chunk[0] != 0)
}

#[cfg(not(windows))]
fn load_windows_credential(_target: &str) -> Result<ApiKey, GatewayError> {
    Err(GatewayError::new(
        GatewayErrorCategory::MissingCredential,
        "当前系统不是Windows，无法读取Windows凭据管理器",
        false,
        "在Windows客户端运行，或在受控服务器部署中显式启用server环境变量模式",
    ))
}

fn missing_key(message: impl Into<String>) -> GatewayError {
    GatewayError::new(
        GatewayErrorCategory::MissingCredential,
        message,
        false,
        "在Windows凭据管理器中保存OpenAI API密钥后重试；不要把密钥写入源码或配置文件",
    )
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
    fn credential_blob_accepts_utf8_and_utf16le() {
        assert_eq!(
            decode_credential_blob(b"fixture-value").expect("utf8"),
            "fixture-value"
        );
        let utf16: Vec<u8> = "fixture-value"
            .encode_utf16()
            .flat_map(u16::to_le_bytes)
            .collect();
        assert_eq!(
            decode_credential_blob(&utf16).expect("utf16"),
            "fixture-value"
        );
    }

    #[test]
    fn credential_target_rejects_path_injection() {
        assert!(
            validate_credential_target("football-match-model-platform/openai/profile-1").is_ok()
        );
        assert!(validate_credential_target("../openai?secret").is_err());
    }
}
