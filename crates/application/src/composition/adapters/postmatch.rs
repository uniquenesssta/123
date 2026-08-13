use super::super::port_registry::PersistenceStore;
use super::map_persistence_error;
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
impl PostmatchSettlementPort for PersistenceStore {
    async fn readiness(&self, review_id: Uuid) -> PortResult<PostmatchSettlementReadiness> {
        self.postmatch_settlement_readiness(review_id)
            .await
            .map_err(map_persistence_error)
    }

    async fn settle(
        &self,
        draft: &PostmatchSettlementDraft,
    ) -> PortResult<PostmatchSettlementRecord> {
        self.settle_postmatch_review(draft)
            .await
            .map_err(map_persistence_error)
    }

    async fn list_settlements(&self, limit: u32) -> PortResult<Vec<PostmatchSettlementRecord>> {
        self.list_postmatch_settlements(limit)
            .await
            .map_err(map_persistence_error)
    }

    async fn list_evidence_scoring_items(
        &self,
        status: Option<&str>,
        limit: u32,
    ) -> PortResult<Vec<EvidenceScoringItemRecord>> {
        self.list_evidence_scoring_items(status, limit)
            .await
            .map_err(map_persistence_error)
    }

    async fn decide_evidence_scoring_item(
        &self,
        draft: &EvidenceScoringDecisionDraft,
    ) -> PortResult<EvidenceScoringItemRecord> {
        self.decide_evidence_scoring_item(draft)
            .await
            .map_err(map_persistence_error)
    }
}

#[async_trait]
impl PostmatchMonitoringPort for PersistenceStore {
    async fn refresh_monitoring(
        &self,
        request: &PostmatchMonitoringRequest,
    ) -> PortResult<PostmatchOverview> {
        self.refresh_postmatch_monitoring(request)
            .await
            .map_err(map_persistence_error)
    }

    async fn overview(&self, limit: u32) -> PortResult<PostmatchOverview> {
        self.postmatch_overview(limit)
            .await
            .map_err(map_persistence_error)
    }
}
