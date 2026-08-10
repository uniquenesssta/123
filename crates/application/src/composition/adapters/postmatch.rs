use super::super::port_registry::{map_persistence_error, ActiveDatabase};
use crate::ports::{
    postmatch::{PostmatchMonitoringPort, PostmatchSettlementPort},
    PortResult,
};
use async_trait::async_trait;
use football_domain::{
    EvidenceScoringDecisionDraft, EvidenceScoringItemRecord, PostmatchMonitoringRequest,
    PostmatchOverview, PostmatchSettlementDraft, PostmatchSettlementReadiness,
    PostmatchSettlementRecord,
};
use uuid::Uuid;

#[async_trait]
impl PostmatchSettlementPort for ActiveDatabase {
    async fn readiness(&self, review_id: Uuid) -> PortResult<PostmatchSettlementReadiness> {
        self.transition_store()
            .postmatch_settlement_readiness(review_id)
            .await
            .map_err(map_persistence_error)
    }

    async fn settle(
        &self,
        draft: &PostmatchSettlementDraft,
    ) -> PortResult<PostmatchSettlementRecord> {
        self.transition_store()
            .settle_postmatch_review(draft)
            .await
            .map_err(map_persistence_error)
    }

    async fn list_settlements(&self, limit: u32) -> PortResult<Vec<PostmatchSettlementRecord>> {
        self.transition_store()
            .list_postmatch_settlements(limit)
            .await
            .map_err(map_persistence_error)
    }

    async fn list_evidence_scoring_items(
        &self,
        status: Option<&str>,
        limit: u32,
    ) -> PortResult<Vec<EvidenceScoringItemRecord>> {
        self.transition_store()
            .list_evidence_scoring_items(status, limit)
            .await
            .map_err(map_persistence_error)
    }

    async fn decide_evidence_scoring_item(
        &self,
        draft: &EvidenceScoringDecisionDraft,
    ) -> PortResult<EvidenceScoringItemRecord> {
        self.transition_store()
            .decide_evidence_scoring_item(draft)
            .await
            .map_err(map_persistence_error)
    }
}

#[async_trait]
impl PostmatchMonitoringPort for ActiveDatabase {
    async fn refresh_monitoring(
        &self,
        request: &PostmatchMonitoringRequest,
    ) -> PortResult<PostmatchOverview> {
        self.transition_store()
            .refresh_postmatch_monitoring(request)
            .await
            .map_err(map_persistence_error)
    }

    async fn overview(&self, limit: u32) -> PortResult<PostmatchOverview> {
        self.transition_store()
            .postmatch_overview(limit)
            .await
            .map_err(map_persistence_error)
    }
}
