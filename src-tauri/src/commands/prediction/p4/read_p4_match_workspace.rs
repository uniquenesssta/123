use crate::{bootstrap::AppState, commands::parse_uuid};
use football_domain::P4MatchWorkspace;
use tauri::State;

#[tauri::command]
pub async fn read_p4_match_workspace(
    state: State<'_, AppState>,
    match_id: String,
) -> Result<P4MatchWorkspace, String> {
    state
        .service
        .read_p4_match_workspace(parse_uuid(&match_id, "比赛 ID")?)
        .await
        .map_err(|error| error.to_string())
}
