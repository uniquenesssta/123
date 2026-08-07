use serde::{Deserialize, Serialize};

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

#[cfg(test)]
mod tests {
    use super::*;

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
