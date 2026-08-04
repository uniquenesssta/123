use super::{ApplicationError, ApplicationResult, ApplicationService};
use football_domain::{
    CompetitionDraft, CompetitionKind, CompetitionRecord, RoundDraft, RoundRecord, SeasonDraft,
    SeasonRecord, StageDraft, StageRecord,
};
use uuid::Uuid;

impl ApplicationService {
    pub async fn create_competition(
        &self,
        mut draft: CompetitionDraft,
    ) -> ApplicationResult<CompetitionRecord> {
        if draft.code.trim().is_empty() {
            let country = draft.country_code.as_deref().unwrap_or("CUSTOM");
            let suffix = Uuid::new_v4().simple().to_string();
            draft.code = format!(
                "{}-{}-{}",
                country.to_ascii_uppercase(),
                compact_key_part(&draft.name),
                &suffix[..8]
            );
        }
        validate_competition(&draft)?;
        let store = self.active_store().await?;
        Ok(store.create_competition(&draft).await?)
    }

    pub async fn delete_competition(&self, competition_id: Uuid) -> ApplicationResult<()> {
        let store = self.active_store().await?;
        Ok(store.delete_competition(competition_id).await?)
    }

    pub async fn create_season(&self, draft: SeasonDraft) -> ApplicationResult<SeasonRecord> {
        validate_season(&draft)?;
        let store = self.active_store().await?;
        Ok(store.create_season(&draft).await?)
    }

    pub async fn create_stage(&self, mut draft: StageDraft) -> ApplicationResult<StageRecord> {
        if draft.code.trim().is_empty() {
            draft.code = format!(
                "STAGE-{}-{}",
                compact_key_part(&draft.name),
                draft.sequence_no
            );
        }
        validate_stage(&draft)?;
        let store = self.active_store().await?;
        Ok(store.create_stage(&draft).await?)
    }

    pub async fn create_round(&self, mut draft: RoundDraft) -> ApplicationResult<RoundRecord> {
        if draft.code.trim().is_empty() {
            draft.code = format!(
                "ROUND-{}-{}",
                compact_key_part(&draft.name),
                draft.sequence_no
            );
        }
        validate_round(&draft)?;
        let store = self.active_store().await?;
        Ok(store.create_round(&draft).await?)
    }
}

fn compact_key_part(value: &str) -> String {
    let normalized: String = value
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric())
        .map(|ch| ch.to_ascii_uppercase())
        .take(12)
        .collect();
    if normalized.is_empty() {
        "TEAM".to_string()
    } else {
        normalized
    }
}

fn validate_competition(draft: &CompetitionDraft) -> ApplicationResult<()> {
    if draft.code.trim().is_empty() || draft.name.trim().is_empty() {
        return Err(ApplicationError::Validation(
            "赛事代码和赛事名称不能为空".to_string(),
        ));
    }
    if draft.timezone.trim().is_empty() {
        return Err(ApplicationError::Validation("赛事时区不能为空".to_string()));
    }
    Ok(())
}

fn validate_season(draft: &SeasonDraft) -> ApplicationResult<()> {
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

fn validate_stage(draft: &StageDraft) -> ApplicationResult<()> {
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

fn validate_round(draft: &RoundDraft) -> ApplicationResult<()> {
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
