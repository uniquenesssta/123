use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum MatchStatus {
    Scheduled,
    Live,
    Finished,
    Postponed,
    Cancelled,
}

impl MatchStatus {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Scheduled => "scheduled",
            Self::Live => "live",
            Self::Finished => "finished",
            Self::Postponed => "postponed",
            Self::Cancelled => "cancelled",
        }
    }
}

impl Default for MatchStatus {
    fn default() -> Self {
        Self::Scheduled
    }
}
