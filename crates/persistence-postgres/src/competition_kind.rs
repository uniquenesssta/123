use crate::{mapping::invalid_state, PersistenceResult};
use football_domain::CompetitionKind;

pub(crate) fn parse_competition_kind(value: &str) -> PersistenceResult<CompetitionKind> {
    match value {
        "league" => Ok(CompetitionKind::League),
        "group_stage" => Ok(CompetitionKind::GroupStage),
        "knockout_single_leg" => Ok(CompetitionKind::KnockoutSingleLeg),
        "knockout_two_leg" => Ok(CompetitionKind::KnockoutTwoLeg),
        "friendly" => Ok(CompetitionKind::Friendly),
        "custom" => Ok(CompetitionKind::Custom),
        other => Err(invalid_state(format!("未知赛事类型：{other}"))),
    }
}
