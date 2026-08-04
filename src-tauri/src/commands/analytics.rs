use super::issues::record_failed_background_jobs;
use super::{parse_uuid, AppState};
use tauri::State;

#[tauri::command]
pub async fn analytics_overview(
    state: State<'_, AppState>,
) -> Result<football_domain::AnalyticsOverview, String> {
    state
        .service
        .analytics_overview()
        .await
        .map_err(|error| error.to_string())
}
#[tauri::command]
pub async fn enqueue_analysis_job(
    state: State<'_, AppState>,
    draft: football_domain::EnqueueJobDraft,
) -> Result<football_domain::BackgroundJob, String> {
    state
        .service
        .enqueue_analysis_job(draft)
        .await
        .map_err(|error| error.to_string())
}
#[tauri::command]
pub async fn list_background_jobs(
    state: State<'_, AppState>,
    limit: u32,
) -> Result<Vec<football_domain::BackgroundJob>, String> {
    let jobs = state
        .service
        .list_background_jobs(limit)
        .await
        .map_err(|error| error.to_string())?;
    record_failed_background_jobs(&state.issue_log, &jobs);
    Ok(jobs)
}
#[tauri::command]
pub async fn cancel_background_job(
    state: State<'_, AppState>,
    job_id: String,
) -> Result<football_domain::BackgroundJob, String> {
    let job_id = parse_uuid(&job_id, "任务 ID")?;
    state
        .service
        .cancel_background_job(job_id)
        .await
        .map_err(|error| error.to_string())
}
#[tauri::command]
pub async fn retry_background_job(
    state: State<'_, AppState>,
    job_id: String,
) -> Result<football_domain::BackgroundJob, String> {
    let job_id = parse_uuid(&job_id, "任务 ID")?;
    state
        .service
        .retry_background_job(job_id)
        .await
        .map_err(|error| error.to_string())
}
#[tauri::command]
pub async fn export_ai_analysis_package(
    state: State<'_, AppState>,
    output_path: String,
) -> Result<football_domain::AiAnalysisPackageSummary, String> {
    state
        .service
        .export_ai_analysis_package(output_path)
        .await
        .map_err(|error| error.to_string())
}
#[tauri::command]
pub fn preview_ai_analysis_response(
    state: State<'_, AppState>,
    input_path: String,
) -> Result<football_domain::AiAnalysisResponsePreview, String> {
    state
        .service
        .preview_ai_analysis_response(input_path)
        .map_err(|error| error.to_string())
}
#[tauri::command]
pub async fn import_ai_analysis_response(
    state: State<'_, AppState>,
    input_path: String,
) -> Result<Vec<football_domain::AiAnalysisSuggestionRecord>, String> {
    state
        .service
        .import_ai_analysis_response(input_path)
        .await
        .map_err(|error| error.to_string())
}
#[tauri::command]
pub async fn list_ai_analysis_suggestions(
    state: State<'_, AppState>,
    status: Option<String>,
    limit: u32,
) -> Result<Vec<football_domain::AiAnalysisSuggestionRecord>, String> {
    state
        .service
        .list_ai_analysis_suggestions(status, limit)
        .await
        .map_err(|error| error.to_string())
}
#[tauri::command]
pub async fn decide_ai_analysis_suggestion(
    state: State<'_, AppState>,
    draft: football_domain::AiSuggestionDecisionDraft,
) -> Result<football_domain::AiAnalysisSuggestionRecord, String> {
    state
        .service
        .decide_ai_analysis_suggestion(draft)
        .await
        .map_err(|error| error.to_string())
}
#[tauri::command]
pub async fn decide_data_quality_finding(
    state: State<'_, AppState>,
    draft: football_domain::DataQualityDecisionDraft,
) -> Result<football_domain::DataQualityFinding, String> {
    state
        .service
        .decide_data_quality_finding(draft)
        .await
        .map_err(|error| error.to_string())
}
#[tauri::command]
pub fn export_ai_analysis_response_template(
    state: State<'_, AppState>,
    output_path: String,
    source_package_id: Option<String>,
) -> Result<String, String> {
    let source_package_id = source_package_id
        .map(|value| parse_uuid(&value, "来源分析包 ID"))
        .transpose()?;
    state
        .service
        .export_ai_analysis_response_template(output_path, source_package_id)
        .map_err(|error| error.to_string())
}
#[tauri::command]
pub async fn generate_parameter_tuning_candidate(
    state: State<'_, AppState>,
    draft: football_domain::ParameterTuningDraft,
) -> Result<football_domain::ParameterTuningCandidateRecord, String> {
    state
        .service
        .generate_parameter_tuning_candidate(draft)
        .await
        .map_err(|error| error.to_string())
}
#[tauri::command]
pub async fn list_parameter_tuning_candidates(
    state: State<'_, AppState>,
    limit: u32,
) -> Result<Vec<football_domain::ParameterTuningCandidateRecord>, String> {
    state
        .service
        .list_parameter_tuning_candidates(limit)
        .await
        .map_err(|error| error.to_string())
}
#[tauri::command]
pub async fn decide_parameter_tuning_candidate(
    state: State<'_, AppState>,
    draft: football_domain::ParameterTuningDecisionDraft,
) -> Result<football_domain::ParameterTuningCandidateRecord, String> {
    state
        .service
        .decide_parameter_tuning_candidate(draft)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn parameter_lifecycle_readiness(
    state: State<'_, AppState>,
    request: football_domain::ParameterLifecycleReadinessRequest,
) -> Result<football_domain::ParameterLifecycleReadiness, String> {
    state
        .service
        .parameter_lifecycle_readiness(request)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn run_parameter_shadow_validation(
    state: State<'_, AppState>,
    request: football_domain::ParameterShadowValidationRequest,
) -> Result<football_domain::ParameterShadowValidationRecord, String> {
    state
        .service
        .run_parameter_shadow_validation(request)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn list_parameter_shadow_validations(
    state: State<'_, AppState>,
    candidate_id: String,
) -> Result<Vec<football_domain::ParameterShadowValidationRecord>, String> {
    state
        .service
        .list_parameter_shadow_validations(parse_uuid(&candidate_id, "候选 ID")?)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn promote_parameter_candidate(
    state: State<'_, AppState>,
    request: football_domain::ParameterPromotionRequest,
) -> Result<football_domain::ParameterPromotionDecisionRecord, String> {
    state
        .service
        .promote_parameter_candidate(request)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn rollback_parameter_candidate(
    state: State<'_, AppState>,
    request: football_domain::ParameterRollbackRequest,
) -> Result<football_domain::ParameterPromotionDecisionRecord, String> {
    state
        .service
        .rollback_parameter_candidate(request)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn list_parameter_promotion_decisions(
    state: State<'_, AppState>,
    candidate_id: String,
) -> Result<Vec<football_domain::ParameterPromotionDecisionRecord>, String> {
    state
        .service
        .list_parameter_promotion_decisions(parse_uuid(&candidate_id, "候选 ID")?)
        .await
        .map_err(|error| error.to_string())
}
