use crate::{ports::review::MatchReviewPort, ApplicationError, ApplicationResult};
use football_domain::{MatchReviewDetail, MatchReviewDraft};

pub(crate) async fn execute<P>(
    port: &P,
    mut draft: MatchReviewDraft,
) -> ApplicationResult<MatchReviewDetail>
where
    P: MatchReviewPort + ?Sized,
{
    if draft.match_id != draft.result.match_id {
        return Err(ApplicationError::Validation(
            "正式赛果与复盘比赛不一致".to_string(),
        ));
    }
    if let Some(version) = draft.review_version.as_mut() {
        *version = version.trim().to_string();
        if version.is_empty() {
            draft.review_version = None;
        }
    }
    if let Some(notes) = draft.notes.as_mut() {
        *notes = notes.trim().to_string();
    }
    Ok(port.generate_match_review(&draft).await?)
}
