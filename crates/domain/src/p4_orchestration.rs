use crate::P4Horizon;
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

pub const P4_ORCHESTRATION_CONTRACT_VERSION: &str = "football.p4-orchestration.v1";
pub const P4_ORCHESTRATION_PLANNER_VERSION: &str = "p4-four-horizon-planner-v1";
pub const P4_RESEARCH_LEAD_MINUTES: i64 = 15;
pub const P4_FREEZE_GRACE_MINUTES: i64 = 15;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum P4FreezeTaskState {
    #[serde(rename = "PLANNED")]
    Planned,
    #[serde(rename = "RESEARCH_QUEUED")]
    ResearchQueued,
    #[serde(rename = "RESEARCH_RUNNING")]
    ResearchRunning,
    #[serde(rename = "RESEARCH_SUCCEEDED")]
    ResearchSucceeded,
    #[serde(rename = "RESEARCH_PARTIAL")]
    ResearchPartial,
    #[serde(rename = "READY_TO_FREEZE")]
    ReadyToFreeze,
    #[serde(rename = "FREEZING")]
    Freezing,
    #[serde(rename = "FROZEN")]
    Frozen,
    #[serde(rename = "BLOCKED")]
    Blocked,
    #[serde(rename = "MISSED")]
    Missed,
    #[serde(rename = "FAILED")]
    Failed,
    #[serde(rename = "CANCELLED")]
    Cancelled,
}

impl P4FreezeTaskState {
    pub const ALL: [Self; 12] = [
        Self::Planned,
        Self::ResearchQueued,
        Self::ResearchRunning,
        Self::ResearchSucceeded,
        Self::ResearchPartial,
        Self::ReadyToFreeze,
        Self::Freezing,
        Self::Frozen,
        Self::Blocked,
        Self::Missed,
        Self::Failed,
        Self::Cancelled,
    ];

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Planned => "PLANNED",
            Self::ResearchQueued => "RESEARCH_QUEUED",
            Self::ResearchRunning => "RESEARCH_RUNNING",
            Self::ResearchSucceeded => "RESEARCH_SUCCEEDED",
            Self::ResearchPartial => "RESEARCH_PARTIAL",
            Self::ReadyToFreeze => "READY_TO_FREEZE",
            Self::Freezing => "FREEZING",
            Self::Frozen => "FROZEN",
            Self::Blocked => "BLOCKED",
            Self::Missed => "MISSED",
            Self::Failed => "FAILED",
            Self::Cancelled => "CANCELLED",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|candidate| candidate.as_str() == value)
    }

    pub const fn is_terminal(self) -> bool {
        matches!(
            self,
            Self::Frozen | Self::Blocked | Self::Missed | Self::Failed | Self::Cancelled
        )
    }

    pub const fn can_transition_to(self, next: Self) -> bool {
        if self as u8 == next as u8 {
            return true;
        }
        matches!(
            (self, next),
            (Self::Planned, Self::ResearchQueued)
                | (Self::Planned, Self::Missed)
                | (Self::Planned, Self::Cancelled)
                | (Self::Planned, Self::Failed)
                | (Self::ResearchQueued, Self::ResearchRunning)
                | (Self::ResearchQueued, Self::Missed)
                | (Self::ResearchQueued, Self::Cancelled)
                | (Self::ResearchQueued, Self::Failed)
                | (Self::ResearchRunning, Self::ResearchSucceeded)
                | (Self::ResearchRunning, Self::ResearchPartial)
                | (Self::ResearchRunning, Self::Missed)
                | (Self::ResearchRunning, Self::Cancelled)
                | (Self::ResearchRunning, Self::Failed)
                | (Self::ResearchSucceeded, Self::ReadyToFreeze)
                | (Self::ResearchSucceeded, Self::Blocked)
                | (Self::ResearchSucceeded, Self::Missed)
                | (Self::ResearchSucceeded, Self::Failed)
                | (Self::ResearchPartial, Self::ResearchSucceeded)
                | (Self::ResearchPartial, Self::Blocked)
                | (Self::ResearchPartial, Self::Cancelled)
                | (Self::ResearchPartial, Self::Failed)
                | (Self::ReadyToFreeze, Self::Freezing)
                | (Self::ReadyToFreeze, Self::Blocked)
                | (Self::ReadyToFreeze, Self::Missed)
                | (Self::ReadyToFreeze, Self::Cancelled)
                | (Self::ReadyToFreeze, Self::Failed)
                | (Self::Freezing, Self::Frozen)
                | (Self::Freezing, Self::Blocked)
                | (Self::Freezing, Self::Missed)
                | (Self::Freezing, Self::Failed)
                | (Self::Blocked, Self::ResearchSucceeded)
        )
    }
}

impl P4Horizon {
    pub const fn offset_minutes(self) -> Option<i64> {
        match self {
            Self::T24h => Some(24 * 60),
            Self::T6h => Some(6 * 60),
            Self::T90m => Some(90),
            Self::T1h => Some(60),
            Self::LegacyTN => None,
        }
    }

    pub fn data_cutoff_at(self, kickoff_at: DateTime<Utc>) -> Option<DateTime<Utc>> {
        self.offset_minutes()
            .map(|minutes| kickoff_at - Duration::minutes(minutes))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct P4PlanningMatchContext {
    pub match_id: Uuid,
    pub match_key: String,
    pub kickoff_at: DateTime<Utc>,
    pub competition_id: Option<Uuid>,
    pub season_id: Option<Uuid>,
    pub stage_id: Option<Uuid>,
    pub competition_kind: crate::CompetitionKind,
    pub home_team_name: String,
    pub away_team_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanP4HorizonsCommand {
    pub match_id: Uuid,
    pub explicit_rule_package_id: Uuid,
    #[serde(default)]
    pub requested_fact_keys: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct P4FreezeTaskDraft {
    pub match_id: Uuid,
    pub match_key: String,
    pub horizon: P4Horizon,
    pub kickoff_at: DateTime<Utc>,
    pub data_cutoff_at: DateTime<Utc>,
    pub research_due_at: DateTime<Utc>,
    pub freeze_deadline_at: DateTime<Utc>,
    pub rule_package_id: Uuid,
    pub model_version_id: Uuid,
    pub parameter_set_id: Uuid,
    pub competition_profile_id: Uuid,
    pub research_schema_version_id: Uuid,
    pub snapshot_schema_version_id: Uuid,
    pub requested_fact_keys: Vec<String>,
    pub trace_id: Uuid,
    pub state: P4FreezeTaskState,
    pub idempotency_key: String,
    #[serde(default)]
    pub metadata: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct P4FreezeTaskRecord {
    pub id: Uuid,
    pub match_id: Uuid,
    pub match_key: String,
    pub horizon: P4Horizon,
    pub kickoff_at: DateTime<Utc>,
    pub data_cutoff_at: DateTime<Utc>,
    pub research_due_at: DateTime<Utc>,
    pub freeze_deadline_at: DateTime<Utc>,
    pub rule_package_id: Uuid,
    pub model_version_id: Uuid,
    pub parameter_set_id: Uuid,
    pub competition_profile_id: Uuid,
    pub research_schema_version_id: Uuid,
    pub snapshot_schema_version_id: Uuid,
    pub requested_fact_keys: Vec<String>,
    pub trace_id: Uuid,
    pub state: P4FreezeTaskState,
    pub research_run_id: Option<Uuid>,
    pub research_job_id: Option<Uuid>,
    pub freeze_job_id: Option<Uuid>,
    pub snapshot_id: Option<Uuid>,
    pub blockers: Value,
    pub task_fingerprint: String,
    pub idempotency_key: String,
    pub metadata: Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct P4FreezeTaskEventRecord {
    pub id: Uuid,
    pub task_id: Uuid,
    pub from_state: Option<P4FreezeTaskState>,
    pub to_state: P4FreezeTaskState,
    pub reason: String,
    pub payload: Value,
    pub occurred_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct P4FreezeReadiness {
    pub task_id: Uuid,
    pub ready: bool,
    pub research_status: Option<String>,
    pub requested_fact_count: u32,
    pub routed_fact_count: u32,
    pub missing_fact_count: u32,
    pub ignored_fact_count: u32,
    pub blocked_fact_count: u32,
    pub blockers: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct P4RoutedFact {
    pub route_key: String,
    pub field_key: String,
    pub target_module: String,
    pub target_slot: String,
    pub route_status: String,
    pub verification_state: String,
    pub selected_evidence_ids: Vec<Uuid>,
    pub selected_value: Value,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct P4FreezeTaskTransition {
    pub task_id: Uuid,
    pub expected_state: P4FreezeTaskState,
    pub next_state: P4FreezeTaskState,
    pub reason: String,
    #[serde(default)]
    pub blockers: Value,
    #[serde(default)]
    pub payload: Value,
    #[serde(default)]
    pub research_run_id: Option<Uuid>,
    #[serde(default)]
    pub research_job_id: Option<Uuid>,
    #[serde(default)]
    pub freeze_job_id: Option<Uuid>,
    #[serde(default)]
    pub snapshot_id: Option<Uuid>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn canonical_horizon_cutoffs_are_exact() {
        let kickoff = Utc
            .with_ymd_and_hms(2026, 7, 20, 12, 0, 0)
            .single()
            .expect("valid kickoff");
        assert_eq!(
            P4Horizon::T24h.data_cutoff_at(kickoff),
            Some(kickoff - Duration::hours(24))
        );
        assert_eq!(
            P4Horizon::T6h.data_cutoff_at(kickoff),
            Some(kickoff - Duration::hours(6))
        );
        assert_eq!(
            P4Horizon::T90m.data_cutoff_at(kickoff),
            Some(kickoff - Duration::minutes(90))
        );
        assert_eq!(
            P4Horizon::T1h.data_cutoff_at(kickoff),
            Some(kickoff - Duration::hours(1))
        );
        assert_eq!(P4Horizon::LegacyTN.data_cutoff_at(kickoff), None);
    }

    #[test]
    fn ready_to_freeze_requires_successful_research_path() {
        assert!(P4FreezeTaskState::ResearchSucceeded
            .can_transition_to(P4FreezeTaskState::ReadyToFreeze));
        assert!(P4FreezeTaskState::ResearchPartial
            .can_transition_to(P4FreezeTaskState::ResearchSucceeded));
        assert!(P4FreezeTaskState::Blocked.can_transition_to(P4FreezeTaskState::ResearchSucceeded));
        assert!(
            !P4FreezeTaskState::ResearchPartial.can_transition_to(P4FreezeTaskState::ReadyToFreeze)
        );
        assert!(P4FreezeTaskState::ReadyToFreeze.can_transition_to(P4FreezeTaskState::Freezing));
        assert!(P4FreezeTaskState::Freezing.can_transition_to(P4FreezeTaskState::Frozen));
        assert!(P4FreezeTaskState::ResearchSucceeded.can_transition_to(P4FreezeTaskState::Missed));
        assert!(P4FreezeTaskState::ReadyToFreeze.can_transition_to(P4FreezeTaskState::Failed));
    }

    #[test]
    fn frozen_task_is_terminal() {
        assert!(P4FreezeTaskState::Frozen.is_terminal());
        assert!(!P4FreezeTaskState::Frozen.can_transition_to(P4FreezeTaskState::Freezing));
    }
}
