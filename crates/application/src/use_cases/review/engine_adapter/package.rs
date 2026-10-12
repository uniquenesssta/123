use crate::{
    ports::review::{MatchReviewPackageStatePort, MatchReviewPort},
    use_cases::review::package::workflow::apply_confirmation_metadata,
    ApplicationError, ApplicationResult,
};
use football_domain::{MatchReviewPackageReviewResult, MatchReviewPackageWorkflowAction};
use uuid::Uuid;

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
