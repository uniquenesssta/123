use crate::{bootstrap::AppState, commands::parse_uuid};
use tauri::State;

#[tauri::command]
pub async fn export_ai_match_package(
    state: State<'_, AppState>,
    output_path: String,
    match_id: String,
) -> Result<football_domain::AiMatchPackageSummary, String> {
    let match_id = parse_uuid(&match_id, "比赛 ID")?;
    state
        .service
        .export_ai_match_package(output_path, match_id)
        .await
        .map_err(|error| error.to_string())
}
