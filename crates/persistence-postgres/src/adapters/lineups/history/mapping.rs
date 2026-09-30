use crate::{
    adapters::catalog::players::value_mapping::availability_status, PersistenceError,
    PersistenceResult,
};
use football_domain::{LineupPlayerRecord, LineupRecord, LineupType};
use sqlx::Row;

fn lineup_type(value: &str) -> PersistenceResult<LineupType> {
    match value {
        "expected" => Ok(LineupType::Expected),
        "confirmed" => Ok(LineupType::Confirmed),
        "actual" => Ok(LineupType::Actual),
        other => Err(PersistenceError::InvalidState(format!(
            "未知阵容类型：{other}"
        ))),
    }
}

pub(crate) fn lineup_player_from_row(
    row: &sqlx::postgres::PgRow,
) -> PersistenceResult<LineupPlayerRecord> {
    let availability: Option<String> = row.try_get("availability_status")?;
    Ok(LineupPlayerRecord {
        player_id: row.try_get("player_id")?,
        player_name: row.try_get("player_name")?,
        position_code: row.try_get("position_code")?,
        role_code: row.try_get("role_code")?,
        role_origin: row.try_get("role_origin")?,
        role_source_position_code: row.try_get("role_source_position_code")?,
        is_starter: row.try_get("is_starter")?,
        shirt_number: row.try_get("shirt_number")?,
        expected_minutes: row.try_get("expected_minutes")?,
        actual_minutes: row.try_get("actual_minutes")?,
        sequence_no: row.try_get("sequence_no")?,
        bench_order: row.try_get("bench_order")?,
        availability_status: availability
            .as_deref()
            .map(availability_status)
            .transpose()?,
        starting_probability: row.try_get("starting_probability")?,
        membership_override: row.try_get("membership_override")?,
        source_urls: row.try_get("source_urls")?,
        validation_warning: row.try_get("validation_warning")?,
    })
}

pub(crate) fn lineup_record_from_row(
    row: &sqlx::postgres::PgRow,
    players: Vec<LineupPlayerRecord>,
) -> PersistenceResult<LineupRecord> {
    let lineup_type_value: String = row.try_get("lineup_type")?;
    Ok(LineupRecord {
        id: row.try_get("id")?,
        match_id: row.try_get("match_id")?,
        match_key: row.try_get("match_key")?,
        team_id: row.try_get("team_id")?,
        team_name: row.try_get("team_name")?,
        lineup_type: lineup_type(&lineup_type_value)?,
        snapshot_type: row.try_get("snapshot_type")?,
        formation: row.try_get("formation")?,
        formation_id: row.try_get("formation_id")?,
        formation_code: row.try_get("formation_code")?,
        formation_name: row.try_get("formation_name")?,
        coach_id: row.try_get("coach_id")?,
        coach_name: row.try_get("coach_name")?,
        captured_at: row.try_get("captured_at")?,
        status: row.try_get("status")?,
        quality_score: row.try_get("quality_score")?,
        source_urls: row.try_get("source_urls")?,
        supersedes_lineup_id: row.try_get("supersedes_lineup_id")?,
        model_validation_status: row.try_get("model_validation_status")?,
        model_eligible: row.try_get("model_eligible")?,
        validation_errors: serde_json::from_value(row.try_get("validation_errors")?)?,
        validation_warnings: serde_json::from_value(row.try_get("validation_warnings")?)?,
        player_count: row.try_get("player_count")?,
        starter_count: row.try_get("starter_count")?,
        players,
    })
}

#[cfg(test)]
mod tests {
    use super::lineup_type;
    use football_domain::LineupType;

    #[test]
    fn persisted_lineup_types_round_trip() {
        assert_eq!(lineup_type("expected").unwrap(), LineupType::Expected);
        assert_eq!(lineup_type("confirmed").unwrap(), LineupType::Confirmed);
        assert_eq!(lineup_type("actual").unwrap(), LineupType::Actual);
    }

    #[test]
    fn unknown_persisted_lineup_type_is_rejected() {
        assert!(lineup_type("draft").is_err());
    }
}
