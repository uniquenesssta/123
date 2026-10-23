use crate::{bootstrap::AppState, commands::parse_uuid};
use tauri::State;

#[tauri::command]
pub async fn delete_competition(
    state: State<'_, AppState>,
    competition_id: String,
) -> Result<(), String> {
    let competition_id = parse_uuid(&competition_id, "赛事 ID")?;
    state
        .service
        .delete_competition(competition_id)
        .await
        .map_err(|error| error.to_string())
}
