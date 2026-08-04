use super::{ApplicationResult, ApplicationService};
use football_domain::{
    EvidenceScoringDecisionDraft, EvidenceScoringItemRecord, PostmatchMonitoringRequest,
    MatchReviewPackageWorkflowAction, MatchReviewPackageWorkflowStatus, PostmatchOverview,
    PostmatchSettlementDraft, PostmatchSettlementReadiness, PostmatchSettlementRecord,
};
use uuid::Uuid;

impl ApplicationService {
    pub async fn postmatch_settlement_readiness(
        &self,
        match_review_id: Uuid,
    ) -> ApplicationResult<PostmatchSettlementReadiness> {
        Ok(self
            .active_store()
            .await?
            .postmatch_settlement_readiness(match_review_id)
            .await?)
    }

    pub async fn settle_postmatch_review(
        &self,
        draft: PostmatchSettlementDraft,
    ) -> ApplicationResult<PostmatchSettlementRecord> {
        let store = self.active_store().await?;
        if let Some(workflow) = store
            .read_match_review_package_workflow_by_review(draft.match_review_id)
            .await?
        {
            if workflow.status != MatchReviewPackageWorkflowStatus::Settled {
                workflow
                    .require_action(MatchReviewPackageWorkflowAction::SettleReview)
                    .map_err(crate::ApplicationError::Validation)?;
            }
        }
        let settlement = store.settle_postmatch_review(&draft).await?;
        store
            .mark_match_review_package_settled(settlement.match_review_id)
            .await?;
        Ok(settlement)
    }

    pub async fn list_postmatch_settlements(
        &self,
        limit: u32,
    ) -> ApplicationResult<Vec<PostmatchSettlementRecord>> {
        Ok(self
            .active_store()
            .await?
            .list_postmatch_settlements(limit)
            .await?)
    }

    pub async fn list_evidence_scoring_items(
        &self,
        status: Option<String>,
        limit: u32,
    ) -> ApplicationResult<Vec<EvidenceScoringItemRecord>> {
        Ok(self
            .active_store()
            .await?
            .list_evidence_scoring_items(status.as_deref(), limit)
            .await?)
    }

    pub async fn decide_evidence_scoring_item(
        &self,
        draft: EvidenceScoringDecisionDraft,
    ) -> ApplicationResult<EvidenceScoringItemRecord> {
        Ok(self
            .active_store()
            .await?
            .decide_evidence_scoring_item(&draft)
            .await?)
    }

    pub async fn refresh_postmatch_monitoring(
        &self,
        request: PostmatchMonitoringRequest,
    ) -> ApplicationResult<PostmatchOverview> {
        Ok(self
            .active_store()
            .await?
            .refresh_postmatch_monitoring(&request)
            .await?)
    }

    pub async fn postmatch_overview(&self, limit: u32) -> ApplicationResult<PostmatchOverview> {
        Ok(self.active_store().await?.postmatch_overview(limit).await?)
    }
}
