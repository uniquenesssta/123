use crate::bootstrap::AppState;
use tauri::State;

#[tauri::command]
pub async fn cancel_api_workspace_request(
    state: State<'_, AppState>,
    request_id: String,
) -> Result<bool, String> {
    let request_id = request_id.trim();
    if request_id.is_empty() {
        return Err("AI问答请求 ID 不能为空".to_string());
    }
    Ok(state.api_workspace_requests.cancel(request_id))
}
