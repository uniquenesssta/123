use super::{parse_uuid, AppState};
use football_application::{
    PredictionCommand, PredictionExecution, RoutePreviewCommand, StoredMatchPredictionCommand,
};
use football_domain::{MatchPredictionReadiness, RouteDecision};
use football_persistence_postgres::ModelRunListItem;
use serde_json::Value;
use tauri::State;

#[tauri::command]
pub fn dry_run_default_fixture(state: State<'_, AppState>) -> Result<Value, String> {
    let output = state
        .service
        .dry_run_default_fixture()
        .map_err(|error| error.to_string())?;
    serde_json::to_value(output).map_err(|error| error.to_string())
}
#[tauri::command]
pub async fn execute_prediction(
    state: State<'_, AppState>,
    command: PredictionCommand,
) -> Result<PredictionExecution, String> {
    state
        .service
        .execute_prediction(command)
        .await
        .map_err(|error| error.to_string())
}
#[tauri::command]
pub async fn inspect_match_prediction_readiness(
    state: State<'_, AppState>,
    command: StoredMatchPredictionCommand,
) -> Result<MatchPredictionReadiness, String> {
    state
        .service
        .inspect_match_prediction_readiness(command)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn execute_prediction_from_match(
    state: State<'_, AppState>,
    command: StoredMatchPredictionCommand,
) -> Result<PredictionExecution, String> {
    state
        .service
        .execute_prediction_from_match(command)
        .await
        .map_err(|error| error.to_string())
}
#[tauri::command]
pub async fn execute_shadow_prediction_from_match(
    state: State<'_, AppState>,
    command: StoredMatchPredictionCommand,
) -> Result<PredictionExecution, String> {
    state
        .service
        .execute_shadow_prediction_from_match(command)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn preview_route(
    state: State<'_, AppState>,
    command: RoutePreviewCommand,
) -> Result<RouteDecision, String> {
    state
        .service
        .preview_route(command)
        .await
        .map_err(|error| error.to_string())
}
#[tauri::command]
pub async fn list_recent_runs(
    state: State<'_, AppState>,
    limit: i64,
) -> Result<Vec<ModelRunListItem>, String> {
    state
        .service
        .list_recent_runs(limit)
        .await
        .map_err(|error| error.to_string())
}
#[tauri::command]
pub async fn read_run(state: State<'_, AppState>, run_id: String) -> Result<Value, String> {
    let parsed = parse_uuid(&run_id, "运行 ID")?;
    state
        .service
        .read_run(parsed)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn hide_model_run_history(
    state: State<'_, AppState>,
    run_id: String,
    reason: Option<String>,
) -> Result<(), String> {
    let parsed = parse_uuid(&run_id, "运行 ID")?;
    state
        .service
        .hide_run_from_history(parsed, reason)
        .await
        .map_err(|error| error.to_string())
}
