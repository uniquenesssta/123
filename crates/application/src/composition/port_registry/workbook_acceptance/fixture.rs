//! Edit explicit QA copies without changing exported source files.
use football_domain::{SpreadsheetEntityType, SpreadsheetImportPreview};
use std::{collections::BTreeSet, path::Path, process::Command};

pub(super) fn enabled_copy(
    source: &str,
    preview: &SpreadsheetImportPreview,
    entity: SpreadsheetEntityType,
    field: &str,
    value: &str,
) -> String {
    let source = Path::new(source);
    let destination = source.with_file_name(format!(
        "{}-update.xlsx",
        source.file_stem().unwrap().to_string_lossy()
    ));
    let sheets = preview
        .rows
        .iter()
        .filter(|row| row.entity_type == entity)
        .map(|row| row.sheet_name.as_str())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>()
        .join("|");
    let script = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src/composition/port_registry/workbook_acceptance/enable-updates.ps1");
    let mut command = Command::new("powershell.exe");
    command
        .args(["-NoProfile", "-ExecutionPolicy", "Bypass", "-File"])
        .arg(script)
        .arg("-Source")
        .arg(source)
        .arg("-Destination")
        .arg(&destination)
        .arg("-DataSheets")
        .arg(sheets)
        .arg("-Field")
        .arg(field)
        .arg("-Value")
        .arg(value);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(0x0800_0000);
    }
    let result = command
        .output()
        .expect("explicit QA workbook action editor");
    assert!(
        result.status.success(),
        "QA action edit failed: {}",
        String::from_utf8_lossy(&result.stderr)
    );
    destination.to_string_lossy().into_owned()
}
