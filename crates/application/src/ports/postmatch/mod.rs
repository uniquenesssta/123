use crate::ports::PortResult;
use async_trait::async_trait;
use football_domain::{
    EvidenceScoringDecisionDraft, EvidenceScoringItemRecord, PostmatchMonitoringRequest,
    PostmatchOverview, PostmatchSettlementDraft, PostmatchSettlementReadiness,
    PostmatchSettlementRecord,
};
use uuid::Uuid;

#[async_trait]
pub trait PostmatchSettlementPort: Send + Sync {
    async fn readiness(&self, review_id: Uuid) -> PortResult<PostmatchSettlementReadiness>;
    async fn settle(
        &self,
        draft: &PostmatchSettlementDraft,
    ) -> PortResult<PostmatchSettlementRecord>;
    async fn list_settlements(&self, limit: u32) -> PortResult<Vec<PostmatchSettlementRecord>>;
    async fn list_evidence_scoring_items(
        &self,
        status: Option<&str>,
        limit: u32,
    ) -> PortResult<Vec<EvidenceScoringItemRecord>>;
    async fn decide_evidence_scoring_item(
        &self,
        draft: &EvidenceScoringDecisionDraft,
    ) -> PortResult<EvidenceScoringItemRecord>;
}

#[async_trait]
pub trait PostmatchMonitoringPort: Send + Sync {
    async fn refresh_monitoring(
        &self,
        request: &PostmatchMonitoringRequest,
    ) -> PortResult<PostmatchOverview>;
    async fn overview(&self, limit: u32) -> PortResult<PostmatchOverview>;
}
