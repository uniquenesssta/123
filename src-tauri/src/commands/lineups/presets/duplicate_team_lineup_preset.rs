use crate::{bootstrap::AppState, commands::parse_uuid};
use football_domain::TeamLineupPresetRecord;
use tauri::State;

#[tauri::command]
pub async fn duplicate_team_lineup_preset(
    state: State<'_, AppState>,
    preset_id: String,
    name: String,
) -> Result<TeamLineupPresetRecord, String> {
    let preset_id = parse_uuid(&preset_id, "阵容预设 ID")?;
    state
        .service
        .duplicate_team_lineup_preset(preset_id, name)
        .await
        .map_err(|error| error.to_string())
}
