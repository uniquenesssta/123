use crate::file_store::{remove_if_exists, write_atomic};
use football_persistence_postgres::DatabaseOptions;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

const CONFIG_VERSION: u32 = 2;
const CREDENTIAL_FILE_NAME: &str = "database.credentials";
const CREDENTIAL_MAGIC: &[u8] = b"FMP-DB-CREDENTIALS-V1\0";

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DesktopConfig {
    pub database: Option<DatabaseOptions>,
}

#[derive(Debug, Serialize, Deserialize)]
struct StoredDesktopConfig {
    version: u32,
    database: Option<StoredDatabaseMetadata>,
}

#[derive(Debug, Serialize, Deserialize)]
struct StoredDatabaseMetadata {
    redacted_url: String,
    max_connections: u32,
    connect_timeout_seconds: u64,
    credential_file: String,
    protection: String,
}

impl DesktopConfig {
    pub fn load(path: &Path) -> Result<Self, String> {
        if !path.exists() {
            return Ok(Self::default());
        }
        let content = fs::read(path)
            .map_err(|error| format!("无法读取配置文件 {}：{error}", path.display()))?;

        if let Ok(stored) = serde_json::from_slice::<StoredDesktopConfig>(&content) {
            return load_stored_config(path, stored);
        }

        let legacy = serde_json::from_slice::<Self>(&content)
            .map_err(|error| format!("配置文件 JSON 无效 {}：{error}", path.display()))?;
        legacy.save(path)?;
        Ok(legacy)
    }

    pub fn save(&self, path: &Path) -> Result<(), String> {
        let stored = if let Some(options) = self.database.as_ref() {
            let serialized = serde_json::to_vec(options)
                .map_err(|error| format!("无法序列化数据库连接配置：{error}"))?;
            let protected = protect_credentials(&serialized)?;
            let mut credential_content =
                Vec::with_capacity(CREDENTIAL_MAGIC.len() + protected.len());
            credential_content.extend_from_slice(CREDENTIAL_MAGIC);
            credential_content.extend_from_slice(&protected);
            write_atomic(&credential_path(path), &credential_content, true)?;

            StoredDesktopConfig {
                version: CONFIG_VERSION,
                database: Some(StoredDatabaseMetadata {
                    redacted_url: options.redacted_url(),
                    max_connections: options.max_connections,
                    connect_timeout_seconds: options.connect_timeout_seconds,
                    credential_file: CREDENTIAL_FILE_NAME.to_string(),
                    protection: protection_name().to_string(),
                }),
            }
        } else {
            StoredDesktopConfig {
                version: CONFIG_VERSION,
                database: None,
            }
        };

        let content = serde_json::to_vec_pretty(&stored)
            .map_err(|error| format!("无法序列化配置：{error}"))?;
        write_atomic(path, &content, true)?;

        if self.database.is_none() {
            remove_if_exists(&credential_path(path))?;
        }
        Ok(())
    }
}

fn load_stored_config(path: &Path, stored: StoredDesktopConfig) -> Result<DesktopConfig, String> {
    if stored.version != CONFIG_VERSION {
        return Err(format!(
            "不支持的配置文件版本 {}，当前支持版本为 {}",
            stored.version, CONFIG_VERSION
        ));
    }
    let Some(metadata) = stored.database else {
        return Ok(DesktopConfig::default());
    };
    if metadata.credential_file != CREDENTIAL_FILE_NAME {
        return Err("数据库凭据文件名称无效，已拒绝读取非预期路径".to_string());
    }
    if metadata.protection != protection_name() {
        return Err(format!(
            "数据库凭据保护方式 {} 与当前系统 {} 不兼容",
            metadata.protection,
            protection_name()
        ));
    }

    let credential_path = credential_path(path);
    let content = fs::read(&credential_path)
        .map_err(|error| format!("无法读取数据库凭据 {}：{error}", credential_path.display()))?;
    let protected = content
        .strip_prefix(CREDENTIAL_MAGIC)
        .ok_or_else(|| "数据库凭据文件格式无效".to_string())?;
    let decrypted = unprotect_credentials(protected)?;
    let options = serde_json::from_slice::<DatabaseOptions>(&decrypted)
        .map_err(|error| format!("数据库凭据内容无效：{error}"))?;

    if options.redacted_url() != metadata.redacted_url
        || options.max_connections != metadata.max_connections
        || options.connect_timeout_seconds != metadata.connect_timeout_seconds
    {
        return Err("数据库连接元数据与受保护凭据不一致，已拒绝自动连接".to_string());
    }

    Ok(DesktopConfig {
        database: Some(options),
    })
}

fn credential_path(config_path: &Path) -> PathBuf {
    config_path.with_file_name(CREDENTIAL_FILE_NAME)
}

#[cfg(windows)]
fn protection_name() -> &'static str {
    "windows-dpapi-current-user"
}

#[cfg(not(windows))]
fn protection_name() -> &'static str {
    "user-private-file"
}

#[cfg(windows)]
mod windows_credentials {
    use std::ffi::c_void;
    use std::ptr::{null, null_mut};

    const CRYPTPROTECT_UI_FORBIDDEN: u32 = 0x0000_0001;

    #[repr(C)]
    struct DataBlob {
        length: u32,
        data: *mut u8,
    }

    #[link(name = "Crypt32")]
    extern "system" {
        fn CryptProtectData(
            data_in: *mut DataBlob,
            description: *const u16,
            optional_entropy: *mut DataBlob,
            reserved: *mut c_void,
            prompt: *mut c_void,
            flags: u32,
            data_out: *mut DataBlob,
        ) -> i32;
        fn CryptUnprotectData(
            data_in: *mut DataBlob,
            description: *mut *mut u16,
            optional_entropy: *mut DataBlob,
            reserved: *mut c_void,
            prompt: *mut c_void,
            flags: u32,
            data_out: *mut DataBlob,
        ) -> i32;
    }

    #[link(name = "Kernel32")]
    extern "system" {
        fn LocalFree(memory: *mut c_void) -> *mut c_void;
    }

    pub(super) fn protect(input: &[u8]) -> Result<Vec<u8>, String> {
        crypt(input, true)
    }

    pub(super) fn unprotect(input: &[u8]) -> Result<Vec<u8>, String> {
        crypt(input, false)
    }

    fn crypt(input: &[u8], protect: bool) -> Result<Vec<u8>, String> {
        let length = u32::try_from(input.len())
            .map_err(|_| "数据库凭据过大，无法使用 Windows DPAPI 保护".to_string())?;
        let mut input_blob = DataBlob {
            length,
            data: input.as_ptr() as *mut u8,
        };
        let mut output_blob = DataBlob {
            length: 0,
            data: null_mut(),
        };

        let succeeded = unsafe {
            if protect {
                CryptProtectData(
                    &mut input_blob,
                    null(),
                    null_mut(),
                    null_mut(),
                    null_mut(),
                    CRYPTPROTECT_UI_FORBIDDEN,
                    &mut output_blob,
                )
            } else {
                CryptUnprotectData(
                    &mut input_blob,
                    null_mut(),
                    null_mut(),
                    null_mut(),
                    null_mut(),
                    CRYPTPROTECT_UI_FORBIDDEN,
                    &mut output_blob,
                )
            }
        };
        if succeeded == 0 {
            return Err(format!(
                "Windows DPAPI {}数据库凭据失败：{}",
                if protect { "保护" } else { "解密" },
                std::io::Error::last_os_error()
            ));
        }
        if output_blob.data.is_null() {
            return Err("Windows DPAPI 返回了空凭据".to_string());
        }

        let output = unsafe {
            let slice = std::slice::from_raw_parts(output_blob.data, output_blob.length as usize);
            let copied = slice.to_vec();
            let _ = LocalFree(output_blob.data.cast::<c_void>());
            copied
        };
        Ok(output)
    }
}

#[cfg(windows)]
fn protect_credentials(input: &[u8]) -> Result<Vec<u8>, String> {
    windows_credentials::protect(input)
}

#[cfg(windows)]
fn unprotect_credentials(input: &[u8]) -> Result<Vec<u8>, String> {
    windows_credentials::unprotect(input)
}

#[cfg(not(windows))]
fn protect_credentials(input: &[u8]) -> Result<Vec<u8>, String> {
    Ok(input.to_vec())
}

#[cfg(not(windows))]
fn unprotect_credentials(input: &[u8]) -> Result<Vec<u8>, String> {
    Ok(input.to_vec())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_options() -> DatabaseOptions {
        DatabaseOptions {
            connection_url: "postgres://football:very-secret@localhost:5432/football".to_string(),
            max_connections: 7,
            connect_timeout_seconds: 15,
        }
    }

    #[test]
    fn saved_config_does_not_contain_plaintext_password() {
        let directory = tempfile::tempdir().expect("创建临时目录");
        let path = directory.path().join("database.json");
        let config = DesktopConfig {
            database: Some(sample_options()),
        };

        config.save(&path).expect("保存配置");
        let metadata = fs::read_to_string(&path).expect("读取元数据");
        assert!(!metadata.contains("very-secret"));
        assert!(metadata.contains("***"));

        let loaded = DesktopConfig::load(&path).expect("读取配置");
        let loaded_options = loaded.database.expect("存在数据库配置");
        assert_eq!(
            loaded_options.connection_url,
            sample_options().connection_url
        );
        assert_eq!(loaded_options.max_connections, 7);
        assert_eq!(loaded_options.connect_timeout_seconds, 15);
    }

    #[test]
    fn legacy_plaintext_config_is_migrated_on_load() {
        let directory = tempfile::tempdir().expect("创建临时目录");
        let path = directory.path().join("database.json");
        let legacy = DesktopConfig {
            database: Some(sample_options()),
        };
        fs::write(
            &path,
            serde_json::to_vec_pretty(&legacy).expect("序列化旧配置"),
        )
        .expect("写入旧配置");

        let loaded = DesktopConfig::load(&path).expect("迁移旧配置");
        assert!(loaded.database.is_some());
        assert!(!fs::read_to_string(&path)
            .expect("读取迁移后配置")
            .contains("very-secret"));
        assert!(credential_path(&path).exists());
    }

    #[test]
    fn clearing_config_removes_credential_file() {
        let directory = tempfile::tempdir().expect("创建临时目录");
        let path = directory.path().join("database.json");
        DesktopConfig {
            database: Some(sample_options()),
        }
        .save(&path)
        .expect("保存配置");

        DesktopConfig::default().save(&path).expect("清除配置");
        assert!(!credential_path(&path).exists());
        assert!(DesktopConfig::load(&path)
            .expect("读取空配置")
            .database
            .is_none());
    }
}
