use crate::{ApplicationError, ApplicationResult};
use football_domain::{ParameterTuningCandidateRecord, ParameterTuningDraft};

pub(crate) async fn execute(
    draft: ParameterTuningDraft,
) -> ApplicationResult<ParameterTuningCandidateRecord> {
    let _ = draft;
    Err(ApplicationError::Model(
        "公开仓库未捆绑模型提供器，不能生成提供器私有参数候选".to_string(),
    ))
}
