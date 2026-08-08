use crate::{ports::competition::CompetitionHierarchyPort, ApplicationError, ApplicationResult};
use football_domain::{SeasonDraft, SeasonRecord};

pub(crate) async fn execute<P>(port: &P, draft: SeasonDraft) -> ApplicationResult<SeasonRecord>
where
    P: CompetitionHierarchyPort + ?Sized,
{
    validate(&draft)?;
    Ok(port.create_season(&draft).await?)
}

fn validate(draft: &SeasonDraft) -> ApplicationResult<()> {
    if draft.name.trim().is_empty() {
        return Err(ApplicationError::Validation("赛季名称不能为空".to_string()));
    }
    if let (Some(starts_on), Some(ends_on)) = (&draft.starts_on, &draft.ends_on) {
        if ends_on < starts_on {
            return Err(ApplicationError::Validation(
                "赛季结束日期不能早于开始日期".to_string(),
            ));
        }
    }
    if !matches!(
        draft.status.as_str(),
        "planned" | "active" | "completed" | "archived"
    ) {
        return Err(ApplicationError::Validation("赛季状态无效".to_string()));
    }
    Ok(())
}
