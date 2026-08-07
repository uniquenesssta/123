use serde::{Deserialize, Serialize};
use super::{EvidenceScoringItemRecord, PostmatchDriftRunRecord, PostmatchSettlementRecord, ProviderScoreSnapshotRecord};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostmatchOverview { pub settlement_count: u64, pub pending_evidence_count: u64, pub scored_evidence_count: u64, pub settlements: Vec<PostmatchSettlementRecord>, pub evidence_queue: Vec<EvidenceScoringItemRecord>, pub provider_scores: Vec<ProviderScoreSnapshotRecord>, pub drift_runs: Vec<PostmatchDriftRunRecord> }
