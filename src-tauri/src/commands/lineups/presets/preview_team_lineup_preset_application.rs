use crate::{bootstrap::AppState, commands::parse_uuid};
use football_domain::TeamLineupPresetApplicationPreview;
use tauri::State;

#[tauri::command]
pub async fn preview_team_lineup_preset_application(
    state: State<'_, AppState>,
    preset_id: String,
) -> Result<TeamLineupPresetApplicationPreview, String> {
    let preset_id = parse_uuid(&preset_id, "阵容预设 ID")?;
    state
        .service
        .preview_team_lineup_preset_application(preset_id)
        .await
        .map_err(|error| error.to_string())
}
