use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, str::FromStr};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash, Default)]
#[serde(rename_all = "snake_case")]
pub enum MatchEventType {
    Substitution,
    Goal,
    OwnGoal,
    Assist,
    PenaltyGoal,
    PenaltyMissed,
    YellowCard,
    SecondYellowCard,
    RedCard,
    Injury,
    Var,
    FormationChange,
    GoalkeeperChange,
    #[default]
    Other,
}

impl MatchEventType {
    pub const ALL: [Self; 14] = [
        Self::Substitution,
        Self::Goal,
        Self::OwnGoal,
        Self::Assist,
        Self::PenaltyGoal,
        Self::PenaltyMissed,
        Self::YellowCard,
        Self::SecondYellowCard,
        Self::RedCard,
        Self::Injury,
        Self::Var,
        Self::FormationChange,
        Self::GoalkeeperChange,
        Self::Other,
    ];

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Substitution => "substitution",
            Self::Goal => "goal",
            Self::OwnGoal => "own_goal",
            Self::Assist => "assist",
            Self::PenaltyGoal => "penalty_goal",
            Self::PenaltyMissed => "penalty_missed",
            Self::YellowCard => "yellow_card",
            Self::SecondYellowCard => "second_yellow_card",
            Self::RedCard => "red_card",
            Self::Injury => "injury",
            Self::Var => "var",
            Self::FormationChange => "formation_change",
            Self::GoalkeeperChange => "goalkeeper_change",
            Self::Other => "other",
        }
    }

    pub const fn counts_toward_score(self) -> bool {
        matches!(self, Self::Goal | Self::OwnGoal | Self::PenaltyGoal)
    }

    pub const fn requires_team(self) -> bool {
        !matches!(self, Self::Var | Self::Other)
    }

    pub const fn requires_player(self) -> bool {
        matches!(
            self,
            Self::Goal
                | Self::OwnGoal
                | Self::Assist
                | Self::PenaltyGoal
                | Self::PenaltyMissed
                | Self::YellowCard
                | Self::SecondYellowCard
                | Self::RedCard
                | Self::Injury
        )
    }
}

impl FromStr for MatchEventType {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.trim().to_ascii_lowercase().as_str() {
            "substitution" => Ok(Self::Substitution),
            "goal" => Ok(Self::Goal),
            "own_goal" | "own-goal" => Ok(Self::OwnGoal),
            "assist" => Ok(Self::Assist),
            "penalty_goal" | "penalty_scored" => Ok(Self::PenaltyGoal),
            "penalty_missed" | "missed_penalty" => Ok(Self::PenaltyMissed),
            "yellow_card" => Ok(Self::YellowCard),
            "second_yellow_card" | "second_yellow" => Ok(Self::SecondYellowCard),
            "red_card" => Ok(Self::RedCard),
            "injury" => Ok(Self::Injury),
            "var" | "var_decision" => Ok(Self::Var),
            "formation_change" => Ok(Self::FormationChange),
            "goalkeeper_change" | "keeper_change" => Ok(Self::GoalkeeperChange),
            "other" => Ok(Self::Other),
            other => Err(format!("未知比赛事件类型：{other}")),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash, Default)]
#[serde(rename_all = "snake_case")]
pub enum MatchEventVerificationStatus {
    #[default]
    Unverified,
    Verified,
    Disputed,
}

impl MatchEventVerificationStatus {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Unverified => "unverified",
            Self::Verified => "verified",
            Self::Disputed => "disputed",
        }
    }
}

impl FromStr for MatchEventVerificationStatus {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.trim().to_ascii_lowercase().as_str() {
            "unverified" | "pending" => Ok(Self::Unverified),
            "verified" | "confirmed" => Ok(Self::Verified),
            "disputed" | "conflict" => Ok(Self::Disputed),
            other => Err(format!("未知事件核验状态：{other}")),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash, Default)]
#[serde(rename_all = "snake_case")]
pub enum MatchEventRevisionStatus {
    #[default]
    Active,
    Corrected,
    Cancelled,
    Superseded,
}

impl MatchEventRevisionStatus {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Corrected => "corrected",
            Self::Cancelled => "cancelled",
            Self::Superseded => "superseded",
        }
    }

    pub const fn is_effective(self) -> bool {
        matches!(self, Self::Active | Self::Corrected)
    }
}

impl FromStr for MatchEventRevisionStatus {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.trim().to_ascii_lowercase().as_str() {
            "active" => Ok(Self::Active),
            "corrected" | "revised" => Ok(Self::Corrected),
            "cancelled" | "canceled" => Ok(Self::Cancelled),
            "superseded" => Ok(Self::Superseded),
            other => Err(format!("未知事件修订状态：{other}")),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MatchEventSummary {
    pub total_count: u64,
    pub effective_count: u64,
    pub cancelled_count: u64,
    pub disputed_count: u64,
    pub verified_count: u64,
    pub event_type_counts: BTreeMap<String, u64>,
    pub latest_home_score: Option<i16>,
    pub latest_away_score: Option<i16>,
    pub last_event_minute: Option<i16>,
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn event_types_parse_and_score_semantics_are_stable() {
        assert_eq!("goal".parse::<MatchEventType>().unwrap(), MatchEventType::Goal);
        assert_eq!(
            "second_yellow".parse::<MatchEventType>().unwrap(),
            MatchEventType::SecondYellowCard
        );
        assert!(MatchEventType::PenaltyGoal.counts_toward_score());
        assert!(MatchEventType::OwnGoal.counts_toward_score());
        assert!(!MatchEventType::Assist.counts_toward_score());
        assert!(MatchEventType::FormationChange.requires_team());
        assert!(!MatchEventType::Var.requires_team());
    }

    #[test]
    fn event_status_defaults_preserve_unverified_legacy_drafts() {
        assert_eq!(
            MatchEventVerificationStatus::default(),
            MatchEventVerificationStatus::Unverified
        );
        assert_eq!(
            MatchEventRevisionStatus::default(),
            MatchEventRevisionStatus::Active
        );
        assert!(MatchEventRevisionStatus::Corrected.is_effective());
        assert!(!MatchEventRevisionStatus::Cancelled.is_effective());
        assert!(!MatchEventRevisionStatus::Superseded.is_effective());
    }
}
