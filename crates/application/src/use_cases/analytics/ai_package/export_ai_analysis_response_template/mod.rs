use crate::ApplicationResult;
use football_analysis_package::response_template_bytes;
use std::fs;
use uuid::Uuid;

pub(crate) fn execute(
    output_path: String,
    source_package_id: Option<Uuid>,
) -> ApplicationResult<String> {
    let bytes = response_template_bytes(source_package_id)?;
    fs::write(&output_path, bytes)?;
    Ok(output_path)
}
