use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum CompetitionKind {
    League,
    GroupStage,
    KnockoutSingleLeg,
    KnockoutTwoLeg,
    Friendly,
    Custom,
}

impl CompetitionKind {
    pub const ALL: [Self; 6] = [
        Self::League,
        Self::GroupStage,
        Self::KnockoutSingleLeg,
        Self::KnockoutTwoLeg,
        Self::Friendly,
        Self::Custom,
    ];

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::League => "league",
            Self::GroupStage => "group_stage",
            Self::KnockoutSingleLeg => "knockout_single_leg",
            Self::KnockoutTwoLeg => "knockout_two_leg",
            Self::Friendly => "friendly",
            Self::Custom => "custom",
        }
    }
}

impl Default for CompetitionKind {
    fn default() -> Self {
        Self::Custom
    }
}
