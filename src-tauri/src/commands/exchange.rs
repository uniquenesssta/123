use super::{parse_uuid, AppState};
use tauri::State;

#[tauri::command]
pub async fn export_team_package_template(
    state: State<'_, AppState>,
    output_path: String,
) -> Result<football_domain::TeamPackageExportSummary, String> {
    state
        .service
        .export_team_package_template(output_path)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn export_team_package_preview_json(
    state: State<'_, AppState>,
    output_path: String,
    preview: football_domain::TeamPackageImportPreview,
) -> Result<football_domain::TeamPackagePreviewExportSummary, String> {
    state
        .service
        .export_team_package_preview_json(output_path, preview)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn preview_team_package_import(
    state: State<'_, AppState>,
    input_path: String,
    mode: football_domain::SpreadsheetImportMode,
) -> Result<football_domain::TeamPackageImportPreview, String> {
    state
        .service
        .preview_team_package_import(input_path, mode)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn commit_team_package_import(
    state: State<'_, AppState>,
    request: football_domain::TeamPackageCommitRequest,
) -> Result<football_domain::TeamPackageCommitResult, String> {
    state
        .service
        .commit_team_package_import(request)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn export_player_catalog_template(
    state: State<'_, AppState>,
    output_path: String,
) -> Result<football_domain::SpreadsheetExportSummary, String> {
    state
        .service
        .export_player_catalog_template(output_path)
        .await
        .map_err(|error| error.to_string())
}
#[tauri::command]
pub async fn export_player_catalog_data(
    state: State<'_, AppState>,
    output_path: String,
) -> Result<football_domain::SpreadsheetExportSummary, String> {
    state
        .service
        .export_player_catalog_data(output_path)
        .await
        .map_err(|error| error.to_string())
}
#[tauri::command]
pub async fn preview_player_catalog_import(
    state: State<'_, AppState>,
    input_path: String,
    mode: football_domain::SpreadsheetImportMode,
) -> Result<football_domain::SpreadsheetImportPreview, String> {
    state
        .service
        .preview_player_catalog_import(input_path, mode)
        .await
        .map_err(|error| error.to_string())
}
#[tauri::command]
pub async fn read_player_catalog_import_preview(
    state: State<'_, AppState>,
    batch_id: String,
) -> Result<football_domain::SpreadsheetImportPreview, String> {
    let parsed = parse_uuid(&batch_id, "批次 ID")?;
    state
        .service
        .read_player_catalog_import_preview(parsed)
        .await
        .map_err(|error| error.to_string())
}
#[tauri::command]
pub async fn resolve_player_catalog_import_conflict(
    state: State<'_, AppState>,
    batch_id: String,
    resolution: football_domain::SpreadsheetImportResolution,
) -> Result<football_domain::SpreadsheetImportPreview, String> {
    let parsed = parse_uuid(&batch_id, "批次 ID")?;
    state
        .service
        .resolve_player_catalog_import_conflict(parsed, resolution)
        .await
        .map_err(|error| error.to_string())
}
#[tauri::command]
pub async fn commit_player_catalog_import(
    state: State<'_, AppState>,
    batch_id: String,
) -> Result<football_domain::SpreadsheetImportCommitResult, String> {
    let parsed = parse_uuid(&batch_id, "批次 ID")?;
    state
        .service
        .commit_player_catalog_import(parsed)
        .await
        .map_err(|error| error.to_string())
}
#[tauri::command]
pub async fn export_match_lineup_template(
    state: State<'_, AppState>,
    output_path: String,
) -> Result<football_domain::MatchLineupExportSummary, String> {
    state
        .service
        .export_match_lineup_template(output_path)
        .await
        .map_err(|error| error.to_string())
}
#[tauri::command]
pub async fn export_match_lineup_data(
    state: State<'_, AppState>,
    output_path: String,
    match_id: String,
) -> Result<football_domain::MatchLineupExportSummary, String> {
    let match_id = parse_uuid(&match_id, "比赛 ID")?;
    state
        .service
        .export_match_lineup_data(output_path, match_id)
        .await
        .map_err(|error| error.to_string())
}
#[tauri::command]
pub async fn preview_match_lineup_import(
    state: State<'_, AppState>,
    input_path: String,
    mode: football_domain::SpreadsheetImportMode,
) -> Result<football_domain::SpreadsheetImportPreview, String> {
    state
        .service
        .preview_match_lineup_import(input_path, mode)
        .await
        .map_err(|error| error.to_string())
}
#[tauri::command]
pub async fn read_match_lineup_import_preview(
    state: State<'_, AppState>,
    batch_id: String,
) -> Result<football_domain::SpreadsheetImportPreview, String> {
    let batch_id = parse_uuid(&batch_id, "批次 ID")?;
    state
        .service
        .read_match_lineup_import_preview(batch_id)
        .await
        .map_err(|error| error.to_string())
}
#[tauri::command]
pub async fn resolve_match_lineup_import_conflict(
    state: State<'_, AppState>,
    batch_id: String,
    resolution: football_domain::SpreadsheetImportResolution,
) -> Result<football_domain::SpreadsheetImportPreview, String> {
    let batch_id = parse_uuid(&batch_id, "批次 ID")?;
    state
        .service
        .resolve_match_lineup_import_conflict(batch_id, resolution)
        .await
        .map_err(|error| error.to_string())
}
#[tauri::command]
pub async fn commit_match_lineup_import(
    state: State<'_, AppState>,
    batch_id: String,
) -> Result<football_domain::SpreadsheetImportCommitResult, String> {
    let batch_id = parse_uuid(&batch_id, "批次 ID")?;
    state
        .service
        .commit_match_lineup_import(batch_id)
        .await
        .map_err(|error| error.to_string())
}
#[tauri::command]
pub async fn export_ai_match_package(
    state: State<'_, AppState>,
    output_path: String,
    match_id: String,
) -> Result<football_domain::AiMatchPackageSummary, String> {
    let match_id = parse_uuid(&match_id, "比赛 ID")?;
    state
        .service
        .export_ai_match_package(output_path, match_id)
        .await
        .map_err(|error| error.to_string())
}
#[tauri::command]
pub async fn preview_ai_match_package(
    state: State<'_, AppState>,
    input_path: String,
    mode: football_domain::SpreadsheetImportMode,
) -> Result<football_domain::SpreadsheetImportPreview, String> {
    state
        .service
        .preview_ai_match_package(input_path, mode)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn export_team_monthly_template(
    state: State<'_, AppState>,
    output_path: String,
) -> Result<football_domain::MonthlyWorkbookExportSummary, String> {
    state
        .service
        .export_team_monthly_template(output_path)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn export_team_monthly_data(
    state: State<'_, AppState>,
    output_path: String,
) -> Result<football_domain::MonthlyWorkbookExportSummary, String> {
    state
        .service
        .export_team_monthly_data(output_path)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn preview_team_monthly_import(
    state: State<'_, AppState>,
    input_path: String,
    mode: football_domain::SpreadsheetImportMode,
) -> Result<football_domain::SpreadsheetImportPreview, String> {
    state
        .service
        .preview_team_monthly_import(input_path, mode)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn read_team_monthly_import_preview(
    state: State<'_, AppState>,
    batch_id: String,
) -> Result<football_domain::SpreadsheetImportPreview, String> {
    let batch_id = parse_uuid(&batch_id, "批次 ID")?;
    state
        .service
        .read_team_monthly_import_preview(batch_id)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn resolve_team_monthly_import_conflict(
    state: State<'_, AppState>,
    batch_id: String,
    resolution: football_domain::SpreadsheetImportResolution,
) -> Result<football_domain::SpreadsheetImportPreview, String> {
    let batch_id = parse_uuid(&batch_id, "批次 ID")?;
    state
        .service
        .resolve_team_monthly_import_conflict(batch_id, resolution)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn commit_team_monthly_import(
    state: State<'_, AppState>,
    batch_id: String,
) -> Result<football_domain::SpreadsheetImportCommitResult, String> {
    let batch_id = parse_uuid(&batch_id, "批次 ID")?;
    state
        .service
        .commit_team_monthly_import(batch_id)
        .await
        .map_err(|error| error.to_string())
}
