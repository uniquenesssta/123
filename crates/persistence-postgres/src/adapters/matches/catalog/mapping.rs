use crate::{PersistenceError, PersistenceResult};
use football_domain::{MatchRecord, MatchStatus};
use sqlx::Row;

fn match_status(value: &str) -> PersistenceResult<MatchStatus> {
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

pub(crate) fn match_record_from_row(row: &sqlx::postgres::PgRow) -> PersistenceResult<MatchRecord> {
    let status: String = row.try_get("status")?;
    Ok(MatchRecord {
        id: row.try_get("id")?,
        external_key: row.try_get("external_key")?,
        competition_id: row.try_get("competition_id")?,
        competition_name: row.try_get("competition_name")?,
        season_id: row.try_get("season_id")?,
        stage_id: row.try_get("stage_id")?,
        round_id: row.try_get("round_id")?,
        home_team_id: row.try_get("home_team_id")?,
        home_team_name: row.try_get("home_team_name")?,
        away_team_id: row.try_get("away_team_id")?,
        away_team_name: row.try_get("away_team_name")?,
        kickoff_time: row.try_get("kickoff_time")?,
        status: match_status(&status)?,
        venue: row.try_get("venue")?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn persisted_match_status_round_trips() {
        assert!(matches!(
            match_status("scheduled").unwrap(),
            MatchStatus::Scheduled
        ));
        assert!(matches!(match_status("live").unwrap(), MatchStatus::Live));
        assert!(matches!(
            match_status("finished").unwrap(),
            MatchStatus::Finished
        ));
        assert!(matches!(
            match_status("postponed").unwrap(),
            MatchStatus::Postponed
        ));
        assert!(matches!(
            match_status("cancelled").unwrap(),
            MatchStatus::Cancelled
        ));
        assert!(match_status("unknown").is_err());
    }
}
