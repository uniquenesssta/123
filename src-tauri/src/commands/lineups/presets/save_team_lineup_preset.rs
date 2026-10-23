use crate::bootstrap::AppState;
use football_domain::{TeamLineupPresetDraft, TeamLineupPresetRecord};
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
