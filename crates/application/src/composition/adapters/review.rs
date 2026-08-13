use super::super::port_registry::PersistenceStore;
use super::map_persistence_error;
use crate::ports::{
    review::{
        MatchReviewPackageFactsPort, MatchReviewPackageSourcePort, MatchReviewPackageStatePort,
        MatchReviewPackageValidationContext, MatchReviewPort,
    },
    PortError, PortErrorKind, PortResult,
};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use football_domain::{
    AbilityCandidateDecisionDraft, AbilityCandidateStatus, AbilityUpdateCandidateRecord,
    MatchReviewDetail, MatchReviewDraft, MatchReviewPackageData, MatchReviewPackagePreview,
    MatchReviewPackageSummary, MatchReviewPackageWorkflowRecord, MatchReviewSummary,
    ReviewableMatch,
};
use serde_json::Value;
use uuid::Uuid;

#[async_trait]
impl MatchReviewPort for PersistenceStore {
    async fn list_reviewable_matches(&self, limit: u32) -> PortResult<Vec<ReviewableMatch>> {
        self.list_reviewable_matches(limit)
            .await
            .map_err(map_persistence_error)
    }

    async fn generate_match_review(
        &self,
        draft: &MatchReviewDraft,
    ) -> PortResult<MatchReviewDetail> {
        self.generate_match_review(draft)
            .await
            .map_err(map_persistence_error)
    }

    async fn list_match_reviews(&self, limit: u32) -> PortResult<Vec<MatchReviewSummary>> {
        self.list_match_reviews(limit)
            .await
            .map_err(map_persistence_error)
    }

    async fn read_match_review(&self, review_id: Uuid) -> PortResult<MatchReviewDetail> {
        self.read_match_review(review_id)
            .await
            .map_err(map_persistence_error)
    }

    async fn list_ability_candidates(
        &self,
        status: Option<AbilityCandidateStatus>,
        limit: u32,
        match_review_id: Option<Uuid>,
    ) -> PortResult<Vec<AbilityUpdateCandidateRecord>> {
        self.list_ability_candidates(status, limit, match_review_id)
            .await
            .map_err(map_persistence_error)
    }

    async fn decide_ability_candidate(
        &self,
        draft: &AbilityCandidateDecisionDraft,
    ) -> PortResult<AbilityUpdateCandidateRecord> {
        self.decide_ability_candidate(draft)
            .await
            .map_err(map_persistence_error)
    }
}

#[async_trait]
impl MatchReviewPackageSourcePort for PersistenceStore {
    async fn build_export_data(
        &self,
        match_id: Uuid,
        package_id: Uuid,
        exported_at: DateTime<Utc>,
    ) -> PortResult<MatchReviewPackageData> {
        let store = PersistenceStore::clone(self);
        let context = store
            .ai_match_package_context(match_id)
            .await
            .map_err(map_persistence_error)?;
        let home_team = store
            .read_team(context.match_record.home_team_id)
            .await
            .map_err(map_persistence_error)?;
        let away_team = store
            .read_team(context.match_record.away_team_id)
            .await
            .map_err(map_persistence_error)?;
        let reviewable = store
            .list_reviewable_matches(200)
            .await
            .map_err(map_persistence_error)?
            .into_iter()
            .find(|item| item.match_record.id == match_id);
        let latest_model_run_id = store
            .list_recent_runs(500)
            .await
            .map_err(map_persistence_error)?
            .into_iter()
            .find(|item| item.match_key == context.match_record.external_key)
            .map(|item| item.id);
        let latest_model_run = match latest_model_run_id {
            Some(run_id) => Some(
                store
                    .read_run(run_id)
                    .await
                    .map_err(map_persistence_error)?,
            ),
            None => None,
        };
        Ok(MatchReviewPackageData {
            package_id,
            exported_at,
            match_record: context.match_record,
            home_team,
            away_team,
            pre_match_lineups: context.lineups,
            player_context: context.players,
            existing_result: reviewable.as_ref().and_then(|item| item.result.clone()),
            latest_review: reviewable.and_then(|item| item.latest_review),
            latest_model_run,
        })
    }

    async fn validation_context(
        &self,
        match_id: Uuid,
    ) -> PortResult<MatchReviewPackageValidationContext> {
        let store = PersistenceStore::clone(self);
        let current_export_data = store
            .match_lineup_export_data(Some(match_id))
            .await
            .map_err(map_persistence_error)?;
        let current_match = current_export_data
            .selected_match
            .clone()
            .ok_or_else(|| PortError::new(PortErrorKind::InvalidState, "资料包关联的比赛不存在"))?;
        let result = store
            .read_match_result(match_id)
            .await
            .map_err(map_persistence_error)?;
        let has_existing_result_or_review = store
            .list_reviewable_matches(200)
            .await
            .map_err(map_persistence_error)?
            .into_iter()
            .find(|item| item.match_record.id == match_id)
            .is_some_and(|item| item.result.is_some() || item.latest_review.is_some());
        let home_team = store
            .read_team(current_match.home_team_id)
            .await
            .map_err(map_persistence_error)?;
        let away_team = store
            .read_team(current_match.away_team_id)
            .await
            .map_err(map_persistence_error)?;
        Ok(MatchReviewPackageValidationContext {
            current_match,
            lineups: current_export_data.lineups,
            result,
            home_registered_player_ids: home_team
                .squad
                .into_iter()
                .map(|item| item.player_id)
                .collect(),
            away_registered_player_ids: away_team
                .squad
                .into_iter()
                .map(|item| item.player_id)
                .collect(),
            has_existing_result_or_review,
        })
    }

    async fn read_source_run(&self, run_id: Uuid) -> PortResult<Value> {
        self.read_run(run_id).await.map_err(map_persistence_error)
    }
}

#[async_trait]
impl MatchReviewPackageStatePort for PersistenceStore {
    async fn register_export(
        &self,
        summary: &MatchReviewPackageSummary,
    ) -> PortResult<MatchReviewPackageWorkflowRecord> {
        self.register_match_review_package_export(summary)
            .await
            .map_err(map_persistence_error)
    }

    async fn read_active_workflow(
        &self,
        match_id: Uuid,
    ) -> PortResult<Option<MatchReviewPackageWorkflowRecord>> {
        self.read_active_match_review_package_workflow(match_id)
            .await
            .map_err(map_persistence_error)
    }

    async fn read_workflow(
        &self,
        package_id: Uuid,
    ) -> PortResult<MatchReviewPackageWorkflowRecord> {
        self.read_match_review_package_workflow(package_id)
            .await
            .map_err(map_persistence_error)
    }

    async fn read_workflow_by_review(
        &self,
        review_id: Uuid,
    ) -> PortResult<Option<MatchReviewPackageWorkflowRecord>> {
        self.read_match_review_package_workflow_by_review(review_id)
            .await
            .map_err(map_persistence_error)
    }

    async fn mark_settled(
        &self,
        review_id: Uuid,
    ) -> PortResult<Option<MatchReviewPackageWorkflowRecord>> {
        self.mark_match_review_package_settled(review_id)
            .await
            .map_err(map_persistence_error)
    }

    async fn record_preview(
        &self,
        package_id: Uuid,
        preview: &MatchReviewPackagePreview,
    ) -> PortResult<MatchReviewPackageWorkflowRecord> {
        self.record_match_review_package_preview(package_id, preview)
            .await
            .map_err(map_persistence_error)
    }

    async fn confirm_workflow(
        &self,
        package_id: Uuid,
        confirmed_by: Option<&str>,
        confirmation_note: Option<&str>,
    ) -> PortResult<MatchReviewPackageWorkflowRecord> {
        self.confirm_match_review_package_workflow(package_id, confirmed_by, confirmation_note)
            .await
            .map_err(map_persistence_error)
    }

    async fn mark_facts_committed(
        &self,
        package_id: Uuid,
    ) -> PortResult<MatchReviewPackageWorkflowRecord> {
        self.mark_match_review_package_facts_committed(package_id)
            .await
            .map_err(map_persistence_error)
    }

    async fn mark_review_created(
        &self,
        package_id: Uuid,
        review_id: Uuid,
    ) -> PortResult<MatchReviewPackageWorkflowRecord> {
        self.mark_match_review_package_review_created(package_id, review_id)
            .await
            .map_err(map_persistence_error)
    }

    async fn read_package_preview(
        &self,
        package_id: Uuid,
    ) -> PortResult<MatchReviewPackagePreview> {
        self.read_match_review_package_preview(package_id)
            .await
            .map_err(map_persistence_error)
    }
}

#[async_trait]
impl MatchReviewPackageFactsPort for PersistenceStore {
    async fn commit_review_facts(&self, draft: &MatchReviewDraft) -> PortResult<()> {
        self.commit_match_review_facts(draft)
            .await
            .map(|_| ())
            .map_err(map_persistence_error)
    }
}
