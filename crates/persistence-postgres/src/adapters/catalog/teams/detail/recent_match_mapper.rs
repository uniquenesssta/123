use super::recent_match_row::TeamRecentMatchRow;
use crate::{PersistenceError, PersistenceResult};
use football_domain::{MatchStatus, TeamRecentMatch};

pub(super) fn map_team_recent_match(row: TeamRecentMatchRow) -> PersistenceResult<TeamRecentMatch> {
    Ok(TeamRecentMatch {
        match_id: row.match_id,
        opponent_team_id: row.opponent_team_id,
        opponent_team_name: row.opponent_team_name,
        kickoff_time: row.kickoff_time,
        venue_side: row.venue_side,
        status: parse_match_status(&row.status)?,
        goals_for: row.goals_for,
        goals_against: row.goals_against,
    })
}

fn parse_match_status(value: &str) -> PersistenceResult<MatchStatus> {
    match value {
        "scheduled" => Ok(MatchStatus::Scheduled),
        "live" => Ok(MatchStatus::Live),
        "finished" => Ok(MatchStatus::Finished),
        "postponed" => Ok(MatchStatus::Postponed),
        "cancelled" => Ok(MatchStatus::Cancelled),
        other => Err(PersistenceError::InvalidState(format!(
            "未知比赛状态：{other}"
        ))),
    }
}
