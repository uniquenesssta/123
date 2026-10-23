use football_application::api_workspace_presets;
use football_domain::ApiWorkspacePreset;

#[tauri::command]
pub fn list_api_workspace_presets() -> Vec<ApiWorkspacePreset> {
    api_workspace_presets()
}
