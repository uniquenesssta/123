use crate::bootstrap::AppState;
use football_domain::{TeamCoachPeriodDraft, TeamCoachPeriodRecord};
use tauri::State;

#[tauri::command]
pub async fn add_team_coach_period(
    state: State<'_, AppState>,
    draft: TeamCoachPeriodDraft,
) -> Result<TeamCoachPeriodRecord, String> {
    state
        .service
        .add_team_coach_period(draft)
        .await
        .map_err(|error| error.to_string())
}
