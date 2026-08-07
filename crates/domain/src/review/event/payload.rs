use super::{MatchEventRevisionStatus, MatchEventType, MatchEventVerificationStatus};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatchReviewEventRecord {
    pub id: Uuid,
    pub match_id: Uuid,
    pub event_key: String,
    pub sequence_no: i32,
    pub event_type: MatchEventType,
    pub team_id: Option<Uuid>,
    pub team_name: Option<String>,
    pub player_id: Option<Uuid>,
    pub player_name: Option<String>,
    pub related_player_id: Option<Uuid>,
    pub related_player_name: Option<String>,
    pub minute: i16,
    pub stoppage_minute: Option<i16>,
    pub period: String,
    pub home_score: Option<i16>,
    pub away_score: Option<i16>,
    pub verification_status: MatchEventVerificationStatus,
    pub revision_status: MatchEventRevisionStatus,
    pub verified_at: Option<DateTime<Utc>>,
    pub source_document_id: Option<Uuid>,
    pub source_package_id: Option<Uuid>,
    pub revision_of_event_id: Option<Uuid>,
    pub description: Option<String>,
    pub source_urls: Vec<String>,
    pub confidence: f64,
    pub metadata: Value,
    pub recorded_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    #[test]
    fn legacy_event_payload_receives_safe_structured_defaults() {
        let event: MatchReviewEventDraft = serde_json::from_value(json!({
            "event_type":"goal","team_id":null,"player_id":null,"related_player_id":null,"minute":12,
            "description":"legacy event","source_urls":[],"confidence":0.8,"metadata":{}
        })).expect("旧赛后资料包事件应继续反序列化");
        assert_eq!(event.event_type, MatchEventType::Goal);
        assert_eq!(
            event.verification_status,
            MatchEventVerificationStatus::Unverified
        );
        assert_eq!(event.revision_status, MatchEventRevisionStatus::Active);
        assert_eq!(event.period, "normal_time");
        assert!(event.event_key.is_none());
        assert!(event.sequence_no.is_none());
    }
}
