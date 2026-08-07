use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MatchReviewPackageSnapshotSummary {
    #[serde(default)] pub home_goals_90: Option<i16>, #[serde(default)] pub away_goals_90: Option<i16>,
    #[serde(default)] pub home_player_count: u64, #[serde(default)] pub away_player_count: u64,
    #[serde(default)] pub home_starter_count: u64, #[serde(default)] pub away_starter_count: u64,
}
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MatchReviewPackageIdentityCheck {
    #[serde(default)] pub package_id_matches_current_export: bool, #[serde(default)] pub match_id_matches_selection: bool,
    #[serde(default)] pub match_key_matches_database: bool, #[serde(default)] pub team_identity_matches_database: bool,
}
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MatchReviewPackageComparison {
    #[serde(default)] pub pre_match: MatchReviewPackageSnapshotSummary,
    #[serde(default)] pub current_database: MatchReviewPackageSnapshotSummary,
    #[serde(default)] pub proposed_import: MatchReviewPackageSnapshotSummary,
    #[serde(default)] pub identity: MatchReviewPackageIdentityCheck,
}
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MatchReviewPackageDiffSummary {
    #[serde(default)] pub home_added_starters: Vec<String>, #[serde(default)] pub home_removed_starters: Vec<String>,
    #[serde(default)] pub away_added_starters: Vec<String>, #[serde(default)] pub away_removed_starters: Vec<String>,
    #[serde(default)] pub added_matchday_players: Vec<String>, #[serde(default)] pub removed_matchday_players: Vec<String>,
}
