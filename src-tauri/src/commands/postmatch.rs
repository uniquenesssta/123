use super::{parse_uuid, AppState};
use tauri::State;

#[tauri::command]
pub async fn postmatch_settlement_readiness(
    state: State<'_, AppState>,
    match_review_id: String,
) -> Result<football_domain::PostmatchSettlementReadiness, String> {
    state
        .service
        .postmatch_settlement_readiness(parse_uuid(&match_review_id, "复盘 ID")?)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn settle_postmatch_review(
    state: State<'_, AppState>,
    draft: football_domain::PostmatchSettlementDraft,
) -> Result<football_domain::PostmatchSettlementRecord, String> {
    state
        .service
        .settle_postmatch_review(draft)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn list_postmatch_settlements(
    state: State<'_, AppState>,
    limit: u32,
) -> Result<Vec<football_domain::PostmatchSettlementRecord>, String> {
    state
        .service
        .list_postmatch_settlements(limit)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn list_evidence_scoring_items(
    state: State<'_, AppState>,
    status: Option<String>,
    limit: u32,
) -> Result<Vec<football_domain::EvidenceScoringItemRecord>, String> {
    state
        .service
        .list_evidence_scoring_items(status, limit)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn decide_evidence_scoring_item(
    state: State<'_, AppState>,
    draft: football_domain::EvidenceScoringDecisionDraft,
) -> Result<football_domain::EvidenceScoringItemRecord, String> {
    state
        .service
        .decide_evidence_scoring_item(draft)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn refresh_postmatch_monitoring(
    state: State<'_, AppState>,
    request: football_domain::PostmatchMonitoringRequest,
) -> Result<football_domain::PostmatchOverview, String> {
    state
        .service
        .refresh_postmatch_monitoring(request)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn postmatch_overview(
    state: State<'_, AppState>,
    limit: u32,
) -> Result<football_domain::PostmatchOverview, String> {
    state
        .service
        .postmatch_overview(limit)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn export_match_review_package(
    state: State<'_, AppState>,
    output_path: String,
    match_id: String,
) -> Result<football_domain::MatchReviewPackageSummary, String> {
    state
        .service
        .export_match_review_package(output_path, parse_uuid(&match_id, "比赛 ID")?)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn preview_match_review_package(
    state: State<'_, AppState>,
    input_path: String,
    expected_match_id: Option<String>,
) -> Result<football_domain::MatchReviewPackagePreview, String> {
    let expected_match_id = expected_match_id
        .as_deref()
        .map(|value| parse_uuid(value, "当前比赛 ID"))
        .transpose()?;
    state
        .service
        .preview_match_review_package(input_path, expected_match_id)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn read_match_review_package_workflow(
    state: State<'_, AppState>,
    match_id: String,
) -> Result<Option<football_domain::MatchReviewPackageWorkflowRecord>, String> {
    state
        .service
        .read_match_review_package_workflow(parse_uuid(&match_id, "比赛 ID")?)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn confirm_match_review_package(
    state: State<'_, AppState>,
    request: football_domain::MatchReviewPackageConfirmationRequest,
) -> Result<football_domain::MatchReviewPackageWorkflowRecord, String> {
    state
        .service
        .confirm_match_review_package(request)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn commit_match_review_package_facts(
    state: State<'_, AppState>,
    package_id: String,
) -> Result<football_domain::MatchReviewPackageFactsCommitResult, String> {
    state
        .service
        .commit_match_review_package_facts(parse_uuid(&package_id, "资料包 ID")?)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn generate_match_review_from_package(
    state: State<'_, AppState>,
    package_id: String,
) -> Result<football_domain::MatchReviewPackageReviewResult, String> {
    state
        .service
        .generate_match_review_from_package(parse_uuid(&package_id, "资料包 ID")?)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn commit_match_review_package(
    state: State<'_, AppState>,
    request: football_domain::MatchReviewPackageCommitRequest,
) -> Result<football_domain::MatchReviewPackageCommitResult, String> {
    state
        .service
        .commit_match_review_package(request)
        .await
        .map_err(|error| error.to_string())
}
