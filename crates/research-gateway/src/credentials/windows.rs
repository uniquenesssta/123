use super::ApiKey;
#[cfg(windows)]
use super::{blob::decode_credential_blob, error::missing_key};
use crate::{GatewayError, GatewayErrorCategory};
#[cfg(windows)]
use zeroize::Zeroize;

#[cfg(windows)]
pub(super) fn load_windows_credential(target: &str) -> Result<ApiKey, GatewayError> {
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
pub(super) fn write_windows_credential(target: &str, api_key: &ApiKey) -> Result<(), GatewayError> {
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
pub(super) fn write_windows_credential(
    _target: &str,
    _api_key: &ApiKey,
) -> Result<(), GatewayError> {
    Err(GatewayError::new(
        GatewayErrorCategory::MissingCredential,
        "当前系统不是Windows，无法保存Windows凭据",
        false,
        "请在Windows桌面客户端中保存OpenAI API密钥",
    ))
}

#[cfg(windows)]
pub(super) fn delete_windows_credential(target: &str) -> Result<(), GatewayError> {
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
pub(super) fn delete_windows_credential(_target: &str) -> Result<(), GatewayError> {
    Err(GatewayError::new(
        GatewayErrorCategory::MissingCredential,
        "当前系统不是Windows，无法删除Windows凭据",
        false,
        "请在Windows桌面客户端中管理OpenAI API密钥",
    ))
}

#[cfg(windows)]
pub(super) fn windows_credential_exists(target: &str) -> Result<bool, GatewayError> {
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
pub(super) fn windows_credential_exists(_target: &str) -> Result<bool, GatewayError> {
    Ok(false)
}

#[cfg(not(windows))]
pub(super) fn load_windows_credential(_target: &str) -> Result<ApiKey, GatewayError> {
    Err(GatewayError::new(
        GatewayErrorCategory::MissingCredential,
        "当前系统不是Windows，无法读取Windows凭据管理器",
        false,
        "在Windows客户端运行，或在受控服务器部署中显式启用server环境变量模式",
    ))
}
