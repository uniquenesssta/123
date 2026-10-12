use crate::{
    ports::ai_workspace::ApiWorkspaceSessionPort, use_cases::ai_workspace::presets,
    ApplicationError, ApplicationResult,
};
use football_domain::{ApiWorkspaceSessionDraft, ApiWorkspaceSessionRecord};
use std::future::Future;

pub(crate) async fn execute<P, F>(
    session: F,
    draft: ApiWorkspaceSessionDraft,
) -> ApplicationResult<ApiWorkspaceSessionRecord>
where
    P: ApiWorkspaceSessionPort,
    F: Future<Output = ApplicationResult<P>>,
{
    let preset = presets::spec(&draft.preset_key)?;
    if preset.preset.requires_match && draft.match_id.is_none() {
        return Err(ApplicationError::Validation(
            "该API协作预设必须选择一场比赛".to_string(),
        ));
    }
    Ok(session.await?.create_session(&draft).await?)
}
