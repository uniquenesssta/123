use crate::{bootstrap::AppState, commands::parse_uuid};
use football_domain::CoachDetail;
use tauri::State;

#[tauri::command]
pub async fn read_coach(
    state: State<'_, AppState>,
    coach_id: String,
) -> Result<CoachDetail, String> {
    let coach_id = parse_uuid(&coach_id, "教练 ID")?;
    state
        .service
        .read_coach(coach_id)
        .await
        .map_err(|error| error.to_string())
}
