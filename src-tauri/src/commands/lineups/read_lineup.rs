use crate::{bootstrap::AppState, commands::parse_uuid};
use football_domain::LineupRecord;
use tauri::State;

#[tauri::command]
pub async fn read_lineup(
    state: State<'_, AppState>,
    lineup_id: String,
) -> Result<LineupRecord, String> {
    let lineup_id = parse_uuid(&lineup_id, "阵容 ID")?;
    state
        .service
        .read_lineup(lineup_id)
        .await
        .map_err(|error| error.to_string())
}
