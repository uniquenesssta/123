use super::{parse_uuid, AppState};
use football_domain::{
    CompetitionBindingDraft, CompetitionBindingSummary, CompetitionDraft, CompetitionRecord,
    RoundDraft, RoundRecord, RulePackageDraft, RulePackageSummary, SeasonDraft, SeasonRecord,
    StageDraft, StageRecord,
};
use tauri::State;

#[tauri::command]
pub async fn create_competition(
    state: State<'_, AppState>,
    draft: CompetitionDraft,
) -> Result<CompetitionRecord, String> {
    state
        .service
        .create_competition(draft)
        .await
        .map_err(|error| error.to_string())
}
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
#[tauri::command]
pub async fn create_season(
    state: State<'_, AppState>,
    draft: SeasonDraft,
) -> Result<SeasonRecord, String> {
    state
        .service
        .create_season(draft)
        .await
        .map_err(|error| error.to_string())
}
#[tauri::command]
pub async fn create_stage(
    state: State<'_, AppState>,
    draft: StageDraft,
) -> Result<StageRecord, String> {
    state
        .service
        .create_stage(draft)
        .await
        .map_err(|error| error.to_string())
}
#[tauri::command]
pub async fn create_round(
    state: State<'_, AppState>,
    draft: RoundDraft,
) -> Result<RoundRecord, String> {
    state
        .service
        .create_round(draft)
        .await
        .map_err(|error| error.to_string())
}
#[tauri::command]
pub async fn register_rule_package(
    state: State<'_, AppState>,
    draft: RulePackageDraft,
) -> Result<RulePackageSummary, String> {
    state
        .service
        .register_rule_package(draft)
        .await
        .map_err(|error| error.to_string())
}
#[tauri::command]
pub async fn create_competition_binding(
    state: State<'_, AppState>,
    draft: CompetitionBindingDraft,
) -> Result<CompetitionBindingSummary, String> {
    state
        .service
        .create_competition_binding(draft)
        .await
        .map_err(|error| error.to_string())
}
