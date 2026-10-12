use crate::{ApplicationError, ApplicationResult};
use football_domain::{
    MatchReviewPackagePreview, MatchReviewPackageWorkflowAction, MatchReviewPackageWorkflowRecord,
};

pub(super) fn matches_revalidated_preview(
    workflow: &MatchReviewPackageWorkflowRecord,
    preview: &MatchReviewPackagePreview,
) -> bool {
    preview.package_id == workflow.package_id
        && preview.source_sha256 == workflow.import_sha256.as_deref().unwrap_or_default()
        && preview.ready
        && preview.errors.is_empty()
}

pub(super) fn ensure_commit_preview_ready(
    preview: &MatchReviewPackagePreview,
) -> ApplicationResult<()> {
    if !preview.ready || !preview.errors.is_empty() {
        return Err(ApplicationError::Validation(
            "赛后复盘资料包仍有阻断错误，不能确认导入".to_string(),
        ));
    }
    Ok(())
}

pub(super) fn ensure_commit_identity(
    workflow: &MatchReviewPackageWorkflowRecord,
    preview: &MatchReviewPackagePreview,
) -> ApplicationResult<()> {
    if workflow
        .require_action(MatchReviewPackageWorkflowAction::ConfirmImport)
        .is_err()
        || workflow.import_sha256.as_deref() != Some(preview.source_sha256.as_str())
        || workflow.match_id != preview.match_id
    {
        return Err(ApplicationError::Validation(
            "当前预检不属于本轮导出的资料包，请重新导出并预检".to_string(),
        ));
    }
    Ok(())
}
