use super::identifiers::compact_key_part;
use crate::{ports::competition::CompetitionHierarchyPort, ApplicationError, ApplicationResult};
use football_domain::{RoundDraft, RoundRecord};

pub(crate) async fn execute<P>(port: &P, mut draft: RoundDraft) -> ApplicationResult<RoundRecord>
where
    P: CompetitionHierarchyPort + ?Sized,
{
    if draft.code.trim().is_empty() {
        draft.code = format!(
            "ROUND-{}-{}",
            compact_key_part(&draft.name),
            draft.sequence_no
        );
    }
    validate(&draft)?;
    Ok(port.create_round(&draft).await?)
}

fn validate(draft: &RoundDraft) -> ApplicationResult<()> {
    if draft.code.trim().is_empty() || draft.name.trim().is_empty() {
        return Err(ApplicationError::Validation(
            "轮次代码和轮次名称不能为空".to_string(),
        ));
    }
    if let (Some(starts_at), Some(ends_at)) = (&draft.starts_at, &draft.ends_at) {
        if ends_at < starts_at {
            return Err(ApplicationError::Validation(
                "轮次结束时间不能早于开始时间".to_string(),
            ));
        }
    }
    Ok(())
}
