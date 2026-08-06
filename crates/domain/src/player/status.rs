use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum PreferredFoot {
    Left,
    Right,
    Both,
    Unknown,
}
impl PreferredFoot {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Left => "left",
            Self::Right => "right",
            Self::Both => "both",
            Self::Unknown => "unknown",
        }
    }
}
impl Default for PreferredFoot {
    fn default() -> Self {
        Self::Unknown
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum PlayerStatus {
    Active,
    Inactive,
    Retired,
    Unknown,
}
impl PlayerStatus {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Inactive => "inactive",
            Self::Retired => "retired",
            Self::Unknown => "unknown",
        }
    }
}
impl Default for PlayerStatus {
    fn default() -> Self {
        Self::Active
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum AvailabilityStatus {
    Available,
    Doubtful,
    Unavailable,
    Injured,
    Suspended,
    Rested,
    Returning,
    Unknown,
}
impl AvailabilityStatus {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Available => "available",
            Self::Doubtful => "doubtful",
            Self::Unavailable => "unavailable",
            Self::Injured => "injured",
            Self::Suspended => "suspended",
            Self::Rested => "rested",
            Self::Returning => "returning",
            Self::Unknown => "unknown",
        }
    }
}
