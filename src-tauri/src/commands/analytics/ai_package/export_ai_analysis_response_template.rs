use crate::{bootstrap::AppState, commands::parse_uuid};
use tauri::State;

#[tauri::command]
pub fn export_ai_analysis_response_template(
    state: State<'_, AppState>,
    output_path: String,
    source_package_id: Option<String>,
) -> Result<String, String> {
    let source_package_id = source_package_id
        .map(|value| parse_uuid(&value, "来源分析包 ID"))
        .transpose()?;
    state
        .service
        .export_ai_analysis_response_template(output_path, source_package_id)
        .map_err(|error| error.to_string())
}
