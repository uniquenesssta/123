use crate::{
    ports::{postmatch::PostmatchSettlementPort, review::MatchReviewPackageStatePort},
    ApplicationError, ApplicationResult,
};
use football_domain::{
    MatchReviewPackageWorkflowAction, MatchReviewPackageWorkflowStatus, PostmatchSettlementDraft,
    PostmatchSettlementRecord,
};

pub(crate) async fn execute<P>(
    port: &P,
    draft: PostmatchSettlementDraft,
) -> ApplicationResult<PostmatchSettlementRecord>
where
    P: PostmatchSettlementPort + MatchReviewPackageStatePort + ?Sized,
{
    if let Some(workflow) = port.read_workflow_by_review(draft.match_review_id).await? {
        if workflow.status != MatchReviewPackageWorkflowStatus::Settled {
            workflow
                .require_action(MatchReviewPackageWorkflowAction::SettleReview)
                .map_err(ApplicationError::Validation)?;
        }
    }
    let settlement = port.settle(&draft).await?;
    port.mark_settled(settlement.match_review_id).await?;
    Ok(settlement)
}
