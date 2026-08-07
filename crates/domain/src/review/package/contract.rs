use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;
use super::MatchReviewPackageSnapshotSummary;
use crate::{AiMatchPlayerContext, LineupRecord, MatchRecord, TeamDetail};
use crate::review::{MatchResultRecord, MatchReviewSummary};

pub const MATCH_REVIEW_PACKAGE_FORMAT: &str = "football.match-review-package.v1";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchReviewPackageData {
    pub package_id: Uuid, pub exported_at: DateTime<Utc>, pub match_record: MatchRecord,
    pub home_team: TeamDetail, pub away_team: TeamDetail,
    #[serde(default)] pub pre_match_lineups: Vec<LineupRecord>,
    #[serde(default)] pub player_context: Vec<AiMatchPlayerContext>,
    #[serde(default)] pub existing_result: Option<MatchResultRecord>,
    #[serde(default)] pub latest_review: Option<MatchReviewSummary>,
    #[serde(default)] pub latest_model_run: Option<Value>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchReviewPackageSummary {
    pub output_path: String, pub package_id: Uuid, pub match_id: Uuid, pub match_key: String,
    pub lineup_count: u64, pub player_count: u64, pub content_sha256: String,
    pub pre_match_snapshot: MatchReviewPackageSnapshotSummary,
    pub export_database_snapshot: MatchReviewPackageSnapshotSummary,
}
