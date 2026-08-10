use crate::ports::PortResult;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use football_domain::{
    LineupRecord, MatchRecord, MatchResultRecord, MatchReviewDraft, MatchReviewPackageData,
    MatchReviewPackagePreview, MatchReviewPackageSummary, MatchReviewPackageWorkflowRecord,
};
use serde_json::Value;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct MatchReviewPackageValidationContext {
    pub current_match: MatchRecord,
    pub lineups: Vec<LineupRecord>,
    pub result: Option<MatchResultRecord>,
    pub home_registered_player_ids: Vec<Uuid>,
    pub away_registered_player_ids: Vec<Uuid>,
    pub has_existing_result_or_review: bool,
}

#[async_trait]
pub trait MatchReviewPackageSourcePort: Send + Sync {
    async fn build_export_data(
        &self,
        match_id: Uuid,
        package_id: Uuid,
        exported_at: DateTime<Utc>,
    ) -> PortResult<MatchReviewPackageData>;

    async fn validation_context(
        &self,
        match_id: Uuid,
    ) -> PortResult<MatchReviewPackageValidationContext>;

    async fn read_source_run(&self, run_id: Uuid) -> PortResult<Value>;
}

#[async_trait]
pub trait MatchReviewPackageStatePort: Send + Sync {
    async fn register_export(
        &self,
        summary: &MatchReviewPackageSummary,
    ) -> PortResult<MatchReviewPackageWorkflowRecord>;

    async fn read_active_workflow(
        &self,
        match_id: Uuid,
    ) -> PortResult<Option<MatchReviewPackageWorkflowRecord>>;

    async fn read_workflow(&self, package_id: Uuid)
        -> PortResult<MatchReviewPackageWorkflowRecord>;

    async fn record_preview(
        &self,
        package_id: Uuid,
        preview: &MatchReviewPackagePreview,
    ) -> PortResult<MatchReviewPackageWorkflowRecord>;

    async fn confirm_workflow(
        &self,
        package_id: Uuid,
        confirmed_by: Option<&str>,
        confirmation_note: Option<&str>,
    ) -> PortResult<MatchReviewPackageWorkflowRecord>;

    async fn mark_facts_committed(
        &self,
        package_id: Uuid,
    ) -> PortResult<MatchReviewPackageWorkflowRecord>;

    async fn mark_review_created(
        &self,
        package_id: Uuid,
        review_id: Uuid,
    ) -> PortResult<MatchReviewPackageWorkflowRecord>;

    async fn read_package_preview(&self, package_id: Uuid)
        -> PortResult<MatchReviewPackagePreview>;
}

#[async_trait]
pub trait MatchReviewPackageFactsPort: Send + Sync {
    async fn commit_review_facts(&self, draft: &MatchReviewDraft) -> PortResult<()>;
}
