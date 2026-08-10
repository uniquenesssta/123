use crate::{ApplicationError, ApplicationResult, ApplicationService};
use football_domain::{
    EvidenceScoringDecisionDraft, EvidenceScoringItemRecord, PostmatchMonitoringRequest,
    PostmatchOverview, PostmatchSettlementDraft, PostmatchSettlementReadiness,
    PostmatchSettlementRecord,
};
use uuid::Uuid;

impl ApplicationService {
    async fn postmatch_session(&self) -> ApplicationResult<crate::composition::ActiveDatabase> {
        self.database
            .active_session()
            .await
            .ok_or(ApplicationError::DatabaseNotConnected)
    }

    pub async fn postmatch_settlement_readiness(
        &self,
        match_review_id: Uuid,
    ) -> ApplicationResult<PostmatchSettlementReadiness> {
        let session = self.postmatch_session().await?;
        self.postmatch
            .postmatch_settlement_readiness(&session, match_review_id)
            .await
    }

    pub async fn settle_postmatch_review(
        &self,
        draft: PostmatchSettlementDraft,
    ) -> ApplicationResult<PostmatchSettlementRecord> {
        let session = self.postmatch_session().await?;
        self.postmatch
            .settle_postmatch_review(&session, draft)
            .await
    }

    pub async fn list_postmatch_settlements(
        &self,
        limit: u32,
    ) -> ApplicationResult<Vec<PostmatchSettlementRecord>> {
        let session = self.postmatch_session().await?;
        self.postmatch
            .list_postmatch_settlements(&session, limit)
            .await
    }

    pub async fn list_evidence_scoring_items(
        &self,
        status: Option<String>,
        limit: u32,
    ) -> ApplicationResult<Vec<EvidenceScoringItemRecord>> {
        let session = self.postmatch_session().await?;
        self.postmatch
            .list_evidence_scoring_items(&session, status, limit)
            .await
    }

    pub async fn decide_evidence_scoring_item(
        &self,
        draft: EvidenceScoringDecisionDraft,
    ) -> ApplicationResult<EvidenceScoringItemRecord> {
        let session = self.postmatch_session().await?;
        self.postmatch
            .decide_evidence_scoring_item(&session, draft)
            .await
    }

    pub async fn refresh_postmatch_monitoring(
        &self,
        request: PostmatchMonitoringRequest,
    ) -> ApplicationResult<PostmatchOverview> {
        let session = self.postmatch_session().await?;
        self.postmatch
            .refresh_postmatch_monitoring(&session, request)
            .await
    }

    pub async fn postmatch_overview(&self, limit: u32) -> ApplicationResult<PostmatchOverview> {
        let session = self.postmatch_session().await?;
        self.postmatch.postmatch_overview(&session, limit).await
    }
}
