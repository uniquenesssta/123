use serde::{Deserialize, Serialize};
use uuid::Uuid;
use super::{MatchReviewPackageComparison, MatchReviewPackageDiffSummary};
use crate::LineupPairDraft;
use crate::review::{MatchReviewDraft, MatchReviewEventDraft};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchReviewPackagePreview {
    pub source_path: String, pub source_file_name: String, pub source_sha256: String, pub format_version: String,
    pub package_id: Uuid, pub match_id: Uuid, pub match_key: String, pub home_team_name: String, pub away_team_name: String,
    pub lineup_pair: LineupPairDraft, pub review: MatchReviewDraft,
    #[serde(default)] pub events: Vec<MatchReviewEventDraft>,
    #[serde(default)] pub comparison: MatchReviewPackageComparison,
    pub diff: MatchReviewPackageDiffSummary,
    #[serde(default)] pub warnings: Vec<String>, #[serde(default)] pub errors: Vec<String>,
    pub home_player_count: u64, pub away_player_count: u64, pub home_starter_count: u64, pub away_starter_count: u64,
    pub substitution_count: u64, pub observation_count: u64, pub ready: bool,
}
