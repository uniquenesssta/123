use super::identifiers::compact_key_part;
use crate::{
    ports::competition::CompetitionHierarchyPort, ApplicationError, ApplicationResult,
};
use football_domain::{CompetitionDraft, CompetitionRecord};
use uuid::Uuid;

pub(crate) async fn execute<P>(
    port: &P,
    mut draft: CompetitionDraft,
) -> ApplicationResult<CompetitionRecord>
where
    P: CompetitionHierarchyPort + ?Sized,
{
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
    validate(&draft)?;
    Ok(port.create_competition(&draft).await?)
}

fn validate(draft: &CompetitionDraft) -> ApplicationResult<()> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ports::{PortError, PortErrorKind, PortResult};
    use async_trait::async_trait;
    use chrono::Utc;
    use football_domain::{
        RoundDraft, RoundRecord, SeasonDraft, SeasonRecord, StageDraft, StageRecord,
    };
    use serde_json::json;

    struct FakeCompetitionPort;

    fn unused<T>() -> PortResult<T> {
        Err(PortError::new(PortErrorKind::Unavailable, "unused fake port method"))
    }

    #[async_trait]
    impl CompetitionHierarchyPort for FakeCompetitionPort {
        async fn create_competition(
            &self,
            draft: &CompetitionDraft,
        ) -> PortResult<CompetitionRecord> {
            Ok(CompetitionRecord {
                id: Uuid::new_v4(),
                code: draft.code.clone(),
                name: draft.name.clone(),
                country_code: draft.country_code.clone(),
                timezone: draft.timezone.clone(),
                competition_kind: draft.competition_kind,
                is_active: true,
                metadata: draft.metadata.clone(),
                created_at: Utc::now(),
            })
        }

        async fn delete_competition(&self, _competition_id: Uuid) -> PortResult<()> {
            unused()
        }

        async fn list_competitions(&self) -> PortResult<Vec<CompetitionRecord>> {
            unused()
        }

        async fn create_season(&self, _draft: &SeasonDraft) -> PortResult<SeasonRecord> {
            unused()
        }

        async fn list_seasons(&self) -> PortResult<Vec<SeasonRecord>> {
            unused()
        }

        async fn create_stage(&self, _draft: &StageDraft) -> PortResult<StageRecord> {
            unused()
        }

        async fn list_stages(&self) -> PortResult<Vec<StageRecord>> {
            unused()
        }

        async fn create_round(&self, _draft: &RoundDraft) -> PortResult<RoundRecord> {
            unused()
        }

        async fn list_rounds(&self) -> PortResult<Vec<RoundRecord>> {
            unused()
        }
    }

    #[tokio::test]
    async fn blank_competition_code_is_generated_before_port_call() {
        let draft = CompetitionDraft {
            code: String::new(),
            name: "Test League".to_string(),
            country_code: Some("us".to_string()),
            timezone: "UTC".to_string(),
            competition_kind: football_domain::CompetitionKind::League,
            metadata: json!({}),
        };
        let created = execute(&FakeCompetitionPort, draft)
            .await
            .expect("赛事创建用例应通过 fake port 独立执行");
        assert!(created.code.starts_with("US-TESTLEAGUE-"));
    }
}
