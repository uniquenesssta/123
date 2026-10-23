use crate::{bootstrap::AppState, commands::parse_uuid};
use football_domain::{
    TeamLineupPresetApplicationPreview, TeamLineupPresetDraft, TeamLineupPresetRecord,
};
use tauri::State;

#[tauri::command]
pub async fn save_team_lineup_preset(
    state: State<'_, AppState>,
    draft: TeamLineupPresetDraft,
) -> Result<TeamLineupPresetRecord, String> {
    state
        .service
        .save_team_lineup_preset(draft)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn list_team_lineup_presets(
    state: State<'_, AppState>,
    team_id: String,
    include_archived: bool,
) -> Result<Vec<TeamLineupPresetRecord>, String> {
    let team_id = parse_uuid(&team_id, "球队 ID")?;
    state
        .service
        .list_team_lineup_presets(team_id, include_archived)
        .await
        .map_err(|error| error.to_string())
}

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

#[tauri::command]
pub async fn archive_team_lineup_preset(
    state: State<'_, AppState>,
    preset_id: String,
) -> Result<TeamLineupPresetRecord, String> {
    let preset_id = parse_uuid(&preset_id, "阵容预设 ID")?;
    state
        .service
        .archive_team_lineup_preset(preset_id)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn delete_team_lineup_preset(
    state: State<'_, AppState>,
    preset_id: String,
) -> Result<(), String> {
    let preset_id = parse_uuid(&preset_id, "阵容预设 ID")?;
    state
        .service
        .delete_team_lineup_preset(preset_id)
        .await
        .map_err(|error| error.to_string())
}
