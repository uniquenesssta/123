use super::identifiers::compact_key_part;
use crate::{
    ports::competition::CompetitionHierarchyPort, ApplicationError, ApplicationResult,
};
use football_domain::{CompetitionKind, StageDraft, StageRecord};

pub(crate) async fn execute<P>(port: &P, mut draft: StageDraft) -> ApplicationResult<StageRecord>
where
    P: CompetitionHierarchyPort + ?Sized,
{
    if draft.code.trim().is_empty() {
        draft.code = format!(
            "STAGE-{}-{}",
            compact_key_part(&draft.name),
            draft.sequence_no
        );
    }
    validate(&draft)?;
    Ok(port.create_stage(&draft).await?)
}

fn validate(draft: &StageDraft) -> ApplicationResult<()> {
    if draft.code.trim().is_empty() || draft.name.trim().is_empty() {
        return Err(ApplicationError::Validation(
            "阶段代码和阶段名称不能为空".to_string(),
        ));
    }
    if draft.stage_kind == CompetitionKind::Friendly {
        return Err(ApplicationError::Validation(
            "友谊赛不能作为赛季阶段类型".to_string(),
        ));
    }
    Ok(())
}
