use crate::{
    ports::{
        lineup::LineupPort,
        review::{
            MatchReviewPackageFactsPort, MatchReviewPackageSourcePort, MatchReviewPackageStatePort,
        },
    },
    use_cases::review::package::{
        preview,
        workflow::{apply_confirmation_metadata, matches_revalidated_preview},
    },
    ApplicationError, ApplicationResult,
};
use football_domain::{MatchReviewPackageFactsCommitResult, MatchReviewPackageWorkflowAction};
use uuid::Uuid;

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
    if !matches_revalidated_preview(&workflow, &preview) {
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
