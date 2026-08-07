use serde::{Deserialize, Serialize};

pub const RELEASE_ACCEPTANCE_CONTRACT_VERSION: &str = "football.integration-j.acceptance.v1";
pub const RELEASE_ACCEPTANCE_FIXTURE_VERSION: &str = "p4-fixed-fixture-v1";

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ReleaseAcceptanceStatus {
    Pass,
    Warning,
    Blocked,
}

impl ReleaseAcceptanceStatus {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Pass => "pass",
            Self::Warning => "warning",
            Self::Blocked => "blocked",
        }
    }
}
