use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum LineupType {
    Expected,
    Confirmed,
    Actual,
}

impl LineupType {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Expected => "expected",
            Self::Confirmed => "confirmed",
            Self::Actual => "actual",
        }
    }
}
