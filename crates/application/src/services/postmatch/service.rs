use crate::{
    ports::{
        postmatch::{PostmatchMonitoringPort, PostmatchSettlementPort},
        review::MatchReviewPackageStatePort,
    },
    use_cases::postmatch,
    ApplicationResult,
};
use football_domain::{
    EvidenceScoringDecisionDraft, EvidenceScoringItemRecord, PostmatchMonitoringRequest,
    PostmatchOverview, PostmatchSettlementDraft, PostmatchSettlementReadiness,
    PostmatchSettlementRecord,
};
use uuid::Uuid;

pub(crate) struct PostmatchService;

impl PostmatchService {
    pub(crate) fn new() -> Self {
        Self
    }

    pub(crate) async fn postmatch_settlement_readiness<P>(
        &self,
        port: &P,
        match_review_id: Uuid,
    ) -> ApplicationResult<PostmatchSettlementReadiness>
    where
        P: PostmatchSettlementPort + ?Sized,
    {
        postmatch::postmatch_settlement_readiness::execute(port, match_review_id).await
    }

    pub(crate) async fn settle_postmatch_review<P>(
        &self,
        port: &P,
        draft: PostmatchSettlementDraft,
    ) -> ApplicationResult<PostmatchSettlementRecord>
    where
        P: PostmatchSettlementPort + MatchReviewPackageStatePort + ?Sized,
    {
        postmatch::settle_postmatch_review::execute(port, draft).await
    }

    pub(crate) async fn list_postmatch_settlements<P>(
        &self,
        port: &P,
        limit: u32,
    ) -> ApplicationResult<Vec<PostmatchSettlementRecord>>
    where
        P: PostmatchSettlementPort + ?Sized,
    {
        postmatch::list_postmatch_settlements::execute(port, limit).await
    }

    pub(crate) async fn list_evidence_scoring_items<P>(
        &self,
        port: &P,
        status: Option<String>,
        limit: u32,
    ) -> ApplicationResult<Vec<EvidenceScoringItemRecord>>
    where
        P: PostmatchSettlementPort + ?Sized,
    {
        postmatch::list_evidence_scoring_items::execute(port, status, limit).await
    }

    pub(crate) async fn decide_evidence_scoring_item<P>(
        &self,
        port: &P,
        draft: EvidenceScoringDecisionDraft,
    ) -> ApplicationResult<EvidenceScoringItemRecord>
    where
        P: PostmatchSettlementPort + ?Sized,
    {
        postmatch::decide_evidence_scoring_item::execute(port, draft).await
    }

    pub(crate) async fn refresh_postmatch_monitoring<P>(
        &self,
        port: &P,
        request: PostmatchMonitoringRequest,
    ) -> ApplicationResult<PostmatchOverview>
    where
        P: PostmatchMonitoringPort + ?Sized,
    {
        postmatch::refresh_postmatch_monitoring::execute(port, request).await
    }

    pub(crate) async fn postmatch_overview<P>(
        &self,
        port: &P,
        limit: u32,
    ) -> ApplicationResult<PostmatchOverview>
    where
        P: PostmatchMonitoringPort + ?Sized,
    {
        postmatch::postmatch_overview::execute(port, limit).await
    }
}
