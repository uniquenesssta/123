use super::{preview, shared::apply_confirmation_metadata};
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
    MatchReviewPackageConfirmationRequest, MatchReviewPackageFactsCommitResult,
    MatchReviewPackageReviewResult, MatchReviewPackageWorkflowAction,
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
    if preview.package_id != workflow.package_id
        || preview.source_sha256 != workflow.import_sha256.as_deref().unwrap_or_default()
        || !preview.ready
        || !preview.errors.is_empty()
    {
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

pub(crate) async fn commit_facts<P>(
    port: &P,
    package_id: Uuid,
) -> ApplicationResult<MatchReviewPackageFactsCommitResult>
where
    P: MatchReviewPackageSourcePort
        + MatchReviewPackageStatePort
        + MatchReviewPackageFactsPort
        + LineupPort
        + ?Sized,
{
    let workflow = port.read_workflow(package_id).await?;
    workflow
        .require_action(MatchReviewPackageWorkflowAction::CommitFacts)
        .map_err(ApplicationError::Validation)?;
    let import_path = workflow.import_path.clone().ok_or_else(|| {
        ApplicationError::Validation("确认记录缺少导入文件路径，请重新预检".to_string())
    })?;
    let mut preview = preview::validate(port, import_path, Some(workflow.match_id), false).await?;
    if preview.package_id != workflow.package_id
        || preview.source_sha256 != workflow.import_sha256.as_deref().unwrap_or_default()
        || !preview.ready
        || !preview.errors.is_empty()
    {
        return Err(ApplicationError::Validation(
            "已确认资料包发生变化，拒绝写入赛后事实".to_string(),
        ));
    }
    apply_confirmation_metadata(&mut preview, &workflow);
    let pair = port.create_lineup_pair(&preview.lineup_pair).await?;
    port.commit_review_facts(&preview.review).await?;
    let workflow = port.mark_facts_committed(package_id).await?;
    Ok(MatchReviewPackageFactsCommitResult {
        home_lineup_id: pair.home.id,
        away_lineup_id: pair.away.id,
        workflow,
    })
}

pub(crate) async fn generate_review<P>(
    port: &P,
    package_id: Uuid,
) -> ApplicationResult<MatchReviewPackageReviewResult>
where
    P: MatchReviewPackageStatePort + MatchReviewPort + ?Sized,
{
    let workflow = port.read_workflow(package_id).await?;
    workflow
        .require_action(MatchReviewPackageWorkflowAction::GenerateReview)
        .map_err(ApplicationError::Validation)?;
    let mut preview = port.read_package_preview(package_id).await?;
    apply_confirmation_metadata(&mut preview, &workflow);
    let review = port.generate_match_review(&preview.review).await?;
    let workflow = port
        .mark_review_created(package_id, review.summary.id)
        .await?;
    Ok(MatchReviewPackageReviewResult { review, workflow })
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
    if !request.preview.ready || !request.preview.errors.is_empty() {
        return Err(ApplicationError::Validation(
            "赛后复盘资料包仍有阻断错误，不能确认导入".to_string(),
        ));
    }
    let workflow = port.read_workflow(request.preview.package_id).await?;
    if workflow
        .require_action(MatchReviewPackageWorkflowAction::ConfirmImport)
        .is_err()
        || workflow.import_sha256.as_deref() != Some(request.preview.source_sha256.as_str())
        || workflow.match_id != request.preview.match_id
    {
        return Err(ApplicationError::Validation(
            "当前预检不属于本轮导出的资料包，请重新导出并预检".to_string(),
        ));
    }
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
