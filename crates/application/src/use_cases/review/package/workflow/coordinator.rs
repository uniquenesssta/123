use super::super::preview;
use super::guards;
use crate::use_cases::review::{engine_adapter::generate_review, facts::commit_facts};
use crate::{
    ports::{
        lineup::LineupPort,
        review::{
            MatchReviewPackageFactsPort, MatchReviewPackageSourcePort, MatchReviewPackageStatePort,
            MatchReviewPort,
        },
    },
    ApplicationError, ApplicationResult,
};
use football_domain::{
    MatchReviewPackageCommitRequest, MatchReviewPackageCommitResult,
    MatchReviewPackageConfirmationRequest, MatchReviewPackageWorkflowAction,
    MatchReviewPackageWorkflowRecord,
};
use uuid::Uuid;

pub(crate) async fn read_active_workflow<P>(
    port: &P,
    match_id: Uuid,
) -> ApplicationResult<Option<MatchReviewPackageWorkflowRecord>>
where
    P: MatchReviewPackageStatePort + ?Sized,
{
    Ok(port.read_active_workflow(match_id).await?)
}

pub(crate) async fn confirm<P>(
    port: &P,
    request: MatchReviewPackageConfirmationRequest,
) -> ApplicationResult<MatchReviewPackageWorkflowRecord>
where
    P: MatchReviewPackageSourcePort + MatchReviewPackageStatePort + ?Sized,
{
    let workflow = port.read_workflow(request.package_id).await?;
    workflow
        .require_action(MatchReviewPackageWorkflowAction::ConfirmImport)
        .map_err(ApplicationError::Validation)?;
    if !workflow.preview_ready {
        return Err(ApplicationError::Validation(
            "本轮导出的资料包尚未通过预检，不能人工确认".to_string(),
        ));
    }
    let import_path = workflow.import_path.clone().ok_or_else(|| {
        ApplicationError::Validation("预检记录缺少导入文件路径，请重新预检".to_string())
    })?;
    let preview = preview::validate(port, import_path, Some(workflow.match_id), false).await?;
    if !guards::matches_revalidated_preview(&workflow, &preview) {
        return Err(ApplicationError::Validation(
            "资料包在预检后已变化或重新校验失败，请重新预检".to_string(),
        ));
    }
    Ok(port
        .confirm_workflow(
            request.package_id,
            request.confirmed_by.as_deref(),
            request.confirmation_note.as_deref(),
        )
        .await?)
}

pub(crate) async fn commit<P>(
    port: &P,
    request: MatchReviewPackageCommitRequest,
) -> ApplicationResult<MatchReviewPackageCommitResult>
where
    P: MatchReviewPackageSourcePort
        + MatchReviewPackageStatePort
        + MatchReviewPackageFactsPort
        + MatchReviewPort
        + LineupPort
        + ?Sized,
{
    guards::ensure_commit_preview_ready(&request.preview)?;
    let workflow = port.read_workflow(request.preview.package_id).await?;
    guards::ensure_commit_identity(&workflow, &request.preview)?;
    let package_id = request.preview.package_id;
    confirm(
        port,
        MatchReviewPackageConfirmationRequest {
            package_id,
            confirmed_by: request.confirmed_by,
            confirmation_note: request.confirmation_note,
        },
    )
    .await?;
    let facts = commit_facts(port, package_id).await?;
    let generated = generate_review(port, package_id).await?;
    Ok(MatchReviewPackageCommitResult {
        home_lineup_id: facts.home_lineup_id,
        away_lineup_id: facts.away_lineup_id,
        review: generated.review,
    })
}
