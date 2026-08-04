use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

use crate::{
    match_review_workflow::{
        MatchReviewPackageActionBlocker, MatchReviewPackageWorkflowAction,
        MatchReviewPackageWorkflowStatus, MatchReviewPackageWorkflowStep,
    },
    AiMatchPlayerContext, LineupPairDraft, LineupRecord, MatchEventRevisionStatus,
    MatchEventType, MatchEventVerificationStatus, MatchRecord, MatchResultRecord,
    MatchReviewDetail, MatchReviewDraft, MatchReviewSummary, TeamDetail,
};

pub const MATCH_REVIEW_PACKAGE_FORMAT: &str = "football.match-review-package.v1";

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MatchReviewPackageSnapshotSummary {
    #[serde(default)]
    pub home_goals_90: Option<i16>,
    #[serde(default)]
    pub away_goals_90: Option<i16>,
    #[serde(default)]
    pub home_player_count: u64,
    #[serde(default)]
    pub away_player_count: u64,
    #[serde(default)]
    pub home_starter_count: u64,
    #[serde(default)]
    pub away_starter_count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MatchReviewPackageIdentityCheck {
    #[serde(default)]
    pub package_id_matches_current_export: bool,
    #[serde(default)]
    pub match_id_matches_selection: bool,
    #[serde(default)]
    pub match_key_matches_database: bool,
    #[serde(default)]
    pub team_identity_matches_database: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MatchReviewPackageComparison {
    #[serde(default)]
    pub pre_match: MatchReviewPackageSnapshotSummary,
    #[serde(default)]
    pub current_database: MatchReviewPackageSnapshotSummary,
    #[serde(default)]
    pub proposed_import: MatchReviewPackageSnapshotSummary,
    #[serde(default)]
    pub identity: MatchReviewPackageIdentityCheck,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchReviewPackageWorkflowRecord {
    pub package_id: Uuid,
    pub match_id: Uuid,
    pub match_key: String,
    pub status: MatchReviewPackageWorkflowStatus,
    #[serde(default)]
    pub completed_steps: Vec<MatchReviewPackageWorkflowStep>,
    #[serde(default)]
    pub allowed_actions: Vec<MatchReviewPackageWorkflowAction>,
    #[serde(default)]
    pub blocking_reasons: Vec<MatchReviewPackageActionBlocker>,
    #[serde(default)]
    pub next_action: Option<MatchReviewPackageWorkflowAction>,
    pub export_path: String,
    pub export_sha256: String,
    pub pre_match_snapshot: MatchReviewPackageSnapshotSummary,
    pub export_database_snapshot: MatchReviewPackageSnapshotSummary,
    pub import_path: Option<String>,
    pub import_sha256: Option<String>,
    pub preview_ready: bool,
    #[serde(default)]
    pub preview: Option<MatchReviewPackagePreview>,
    pub confirmed_by: Option<String>,
    pub confirmation_note: Option<String>,
    pub review_id: Option<Uuid>,
    pub exported_at: DateTime<Utc>,
    pub previewed_at: Option<DateTime<Utc>>,
    pub confirmed_at: Option<DateTime<Utc>>,
    pub facts_committed_at: Option<DateTime<Utc>>,
    pub review_created_at: Option<DateTime<Utc>>,
    pub settled_at: Option<DateTime<Utc>>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchReviewPackageConfirmationRequest {
    pub package_id: Uuid,
    #[serde(default)]
    pub confirmed_by: Option<String>,
    #[serde(default)]
    pub confirmation_note: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchReviewPackageFactsCommitResult {
    pub home_lineup_id: Uuid,
    pub away_lineup_id: Uuid,
    pub workflow: MatchReviewPackageWorkflowRecord,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchReviewPackageReviewResult {
    pub review: MatchReviewDetail,
    pub workflow: MatchReviewPackageWorkflowRecord,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchReviewPackageData {
    pub package_id: Uuid,
    pub exported_at: DateTime<Utc>,
    pub match_record: MatchRecord,
    pub home_team: TeamDetail,
    pub away_team: TeamDetail,
    #[serde(default)]
    pub pre_match_lineups: Vec<LineupRecord>,
    #[serde(default)]
    pub player_context: Vec<AiMatchPlayerContext>,
    #[serde(default)]
    pub existing_result: Option<MatchResultRecord>,
    #[serde(default)]
    pub latest_review: Option<MatchReviewSummary>,
    #[serde(default)]
    pub latest_model_run: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchReviewPackageSummary {
    pub output_path: String,
    pub package_id: Uuid,
    pub match_id: Uuid,
    pub match_key: String,
    pub lineup_count: u64,
    pub player_count: u64,
    pub content_sha256: String,
    pub pre_match_snapshot: MatchReviewPackageSnapshotSummary,
    pub export_database_snapshot: MatchReviewPackageSnapshotSummary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchReviewEventDraft {
    #[serde(default)]
    pub event_key: Option<String>,
    #[serde(default)]
    pub sequence_no: Option<i32>,
    pub event_type: MatchEventType,
    #[serde(default)]
    pub team_id: Option<Uuid>,
    #[serde(default)]
    pub player_id: Option<Uuid>,
    #[serde(default)]
    pub related_player_id: Option<Uuid>,
    pub minute: i16,
    #[serde(default)]
    pub stoppage_minute: Option<i16>,
    #[serde(default = "default_event_period")]
    pub period: String,
    #[serde(default)]
    pub home_score: Option<i16>,
    #[serde(default)]
    pub away_score: Option<i16>,
    #[serde(default)]
    pub verification_status: MatchEventVerificationStatus,
    #[serde(default)]
    pub revision_status: MatchEventRevisionStatus,
    #[serde(default)]
    pub verified_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub source_document_id: Option<Uuid>,
    #[serde(default)]
    pub source_package_id: Option<Uuid>,
    #[serde(default)]
    pub revision_of_event_id: Option<Uuid>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub source_urls: Vec<String>,
    #[serde(default = "default_event_confidence")]
    pub confidence: f64,
    #[serde(default)]
    pub metadata: Value,
}

fn default_event_period() -> String {
    "normal_time".to_string()
}

fn default_event_confidence() -> f64 {
    1.0
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MatchReviewPackageDiffSummary {
    #[serde(default)]
    pub home_added_starters: Vec<String>,
    #[serde(default)]
    pub home_removed_starters: Vec<String>,
    #[serde(default)]
    pub away_added_starters: Vec<String>,
    #[serde(default)]
    pub away_removed_starters: Vec<String>,
    #[serde(default)]
    pub added_matchday_players: Vec<String>,
    #[serde(default)]
    pub removed_matchday_players: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchReviewPackagePreview {
    pub source_path: String,
    pub source_file_name: String,
    pub source_sha256: String,
    pub format_version: String,
    pub package_id: Uuid,
    pub match_id: Uuid,
    pub match_key: String,
    pub home_team_name: String,
    pub away_team_name: String,
    pub lineup_pair: LineupPairDraft,
    pub review: MatchReviewDraft,
    #[serde(default)]
    pub events: Vec<MatchReviewEventDraft>,
    #[serde(default)]
    pub comparison: MatchReviewPackageComparison,
    pub diff: MatchReviewPackageDiffSummary,
    #[serde(default)]
    pub warnings: Vec<String>,
    #[serde(default)]
    pub errors: Vec<String>,
    pub home_player_count: u64,
    pub away_player_count: u64,
    pub home_starter_count: u64,
    pub away_starter_count: u64,
    pub substitution_count: u64,
    pub observation_count: u64,
    pub ready: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchReviewPackageCommitRequest {
    pub preview: MatchReviewPackagePreview,
    #[serde(default)]
    pub confirmed_by: Option<String>,
    #[serde(default)]
    pub confirmation_note: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchReviewPackageCommitResult {
    pub home_lineup_id: Uuid,
    pub away_lineup_id: Uuid,
    pub review: MatchReviewDetail,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn legacy_event_payload_receives_safe_structured_defaults() {
        let event: MatchReviewEventDraft = serde_json::from_value(json!({
            "event_type": "goal",
            "team_id": null,
            "player_id": null,
            "related_player_id": null,
            "minute": 12,
            "description": "legacy event",
            "source_urls": [],
            "confidence": 0.8,
            "metadata": {}
        }))
        .expect("旧赛后资料包事件应继续反序列化");

        assert_eq!(event.event_type, MatchEventType::Goal);
        assert_eq!(event.verification_status, MatchEventVerificationStatus::Unverified);
        assert_eq!(event.revision_status, MatchEventRevisionStatus::Active);
        assert_eq!(event.period, "normal_time");
        assert!(event.event_key.is_none());
        assert!(event.sequence_no.is_none());
    }
}
