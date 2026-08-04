use std::fs::{self, File, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_TEMP_ID: AtomicU64 = AtomicU64::new(1);

pub fn write_atomic(path: &Path, content: &[u8], private: bool) -> Result<(), String> {
    let parent = path
        .parent()
        .ok_or_else(|| format!("目标文件没有父目录：{}", path.display()))?;
    fs::create_dir_all(parent)
        .map_err(|error| format!("无法创建目录 {}：{error}", parent.display()))?;

    let temp_path = temporary_path(path);
    let result = (|| -> Result<(), String> {
        let mut file = open_temp_file(&temp_path, private)
            .map_err(|error| format!("无法创建临时文件 {}：{error}", temp_path.display()))?;
        file.write_all(content)
            .map_err(|error| format!("无法写入临时文件 {}：{error}", temp_path.display()))?;
        file.flush()
            .map_err(|error| format!("无法刷新临时文件 {}：{error}", temp_path.display()))?;
        file.sync_all()
            .map_err(|error| format!("无法同步临时文件 {}：{error}", temp_path.display()))?;
        drop(file);

        replace_file(&temp_path, path)
            .map_err(|error| format!("无法原子替换文件 {}：{error}", path.display()))?;
        sync_parent_directory(parent);
        Ok(())
    })();

    if result.is_err() {
        let _ = fs::remove_file(&temp_path);
    }
    result
}

pub fn remove_if_exists(path: &Path) -> Result<(), String> {
    match fs::remove_file(path) {
        Ok(()) => {
            if let Some(parent) = path.parent() {
                sync_parent_directory(parent);
            }
            Ok(())
        }
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(format!("无法删除文件 {}：{error}", path.display())),
    }
}

fn temporary_path(path: &Path) -> PathBuf {
    let file_name = path
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("data");
    let sequence = NEXT_TEMP_ID.fetch_add(1, Ordering::Relaxed);
    path.with_file_name(format!(
        ".{file_name}.{}.{}.tmp",
        std::process::id(),
        sequence
    ))
}

#[cfg(unix)]
fn open_temp_file(path: &Path, private: bool) -> io::Result<File> {
    use std::os::unix::fs::OpenOptionsExt;

    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    if private {
        options.mode(0o600);
    }
    options.open(path)
}

#[cfg(not(unix))]
fn open_temp_file(path: &Path, _private: bool) -> io::Result<File> {
    OpenOptions::new().write(true).create_new(true).open(path)
}

#[cfg(windows)]
mod windows_replace {
    use std::io;
    use std::os::windows::ffi::OsStrExt;
    use std::path::Path;

    const MOVEFILE_REPLACE_EXISTING: u32 = 0x0000_0001;
    const MOVEFILE_WRITE_THROUGH: u32 = 0x0000_0008;

    #[link(name = "Kernel32")]
    extern "system" {
        fn MoveFileExW(
            existing_file_name: *const u16,
            new_file_name: *const u16,
            flags: u32,
        ) -> i32;
    }

    pub(super) fn replace_file(source: &Path, destination: &Path) -> io::Result<()> {
        let source_wide: Vec<u16> = source.as_os_str().encode_wide().chain(Some(0)).collect();
        let destination_wide: Vec<u16> = destination
            .as_os_str()
            .encode_wide()
            .chain(Some(0))
            .collect();
        let succeeded = unsafe {
            MoveFileExW(
                source_wide.as_ptr(),
                destination_wide.as_ptr(),
                MOVEFILE_REPLACE_EXISTING | MOVEFILE_WRITE_THROUGH,
            )
        };
        if succeeded == 0 {
            Err(io::Error::last_os_error())
        } else {
            Ok(())
        }
    }
}

#[cfg(windows)]
fn replace_file(source: &Path, destination: &Path) -> io::Result<()> {
    windows_replace::replace_file(source, destination)
}

#[cfg(not(windows))]
fn replace_file(source: &Path, destination: &Path) -> io::Result<()> {
    fs::rename(source, destination)
}

#[cfg(unix)]
fn sync_parent_directory(path: &Path) {
    if let Ok(directory) = File::open(path) {
        let _ = directory.sync_all();
    }
}

#[cfg(not(unix))]
fn sync_parent_directory(_path: &Path) {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn atomic_write_replaces_existing_content() {
        let directory = tempfile::tempdir().expect("创建临时目录");
        let path = directory.path().join("state.json");

        write_atomic(&path, b"first", true).expect("首次写入");
        write_atomic(&path, b"second", true).expect("替换写入");

        assert_eq!(fs::read(&path).expect("读取文件"), b"second");
        assert!(directory.path().read_dir().expect("读取目录").all(|entry| {
            !entry
                .expect("读取目录项")
                .file_name()
                .to_string_lossy()
                .ends_with(".tmp")
        }));
    }

    #[test]
    fn remove_missing_file_is_idempotent() {
        let directory = tempfile::tempdir().expect("创建临时目录");
        let path = directory.path().join("missing.json");
        remove_if_exists(&path).expect("删除不存在文件");
    }
}
