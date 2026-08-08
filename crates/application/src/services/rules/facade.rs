use crate::{ApplicationError, ApplicationResult, ApplicationService};
use football_domain::{
    CompetitionBindingDraft, CompetitionBindingSummary, RulePackageDraft, RulePackageSummary,
};

impl ApplicationService {
    pub async fn register_rule_package(
        &self,
        draft: RulePackageDraft,
    ) -> ApplicationResult<RulePackageSummary> {
        let session = self
            .database
            .active_session()
            .await
            .ok_or(ApplicationError::DatabaseNotConnected)?;
        self.rules
            .register_package(&self.registry, &session, draft)
            .await
    }

    pub async fn create_competition_binding(
        &self,
        draft: CompetitionBindingDraft,
    ) -> ApplicationResult<CompetitionBindingSummary> {
        let session = self
            .database
            .active_session()
            .await
            .ok_or(ApplicationError::DatabaseNotConnected)?;
        self.rules.create_binding(&session, draft).await
    }
}
