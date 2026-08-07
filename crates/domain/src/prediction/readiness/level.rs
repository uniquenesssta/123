use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PredictionReadinessLevel {
    FormalReady,
    ReadyWithWarnings,
    ShadowOnly,
    Blocked,
}

impl PredictionReadinessLevel {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::FormalReady => "formal_ready",
            Self::ReadyWithWarnings => "ready_with_warnings",
            Self::ShadowOnly => "shadow_only",
            Self::Blocked => "blocked",
        }
    }

    pub const fn can_run_formal(self) -> bool {
        matches!(self, Self::FormalReady | Self::ReadyWithWarnings)
    }

    pub const fn can_run_shadow(self) -> bool {
        !matches!(self, Self::Blocked)
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PredictionReadinessCheckStatus {
    Passed,
    Warning,
    Blocked,
}
