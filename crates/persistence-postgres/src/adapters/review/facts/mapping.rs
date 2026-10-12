use crate::{PersistenceError, PersistenceResult};
use football_domain::{
    MatchEventRevisionStatus, MatchEventType, MatchEventVerificationStatus, MatchResultRecord,
    MatchReviewEventRecord, SubstitutionRecord,
};
use sqlx::Row;

pub(super) fn match_result_from_row(
    row: &sqlx::postgres::PgRow,
) -> PersistenceResult<MatchResultRecord> {
    Ok(MatchResultRecord {
        match_id: row.try_get("match_id")?,
        home_goals_90: row.try_get("home_goals_90")?,
        away_goals_90: row.try_get("away_goals_90")?,
        home_goals_extra_time: row.try_get("home_goals_extra_time")?,
        away_goals_extra_time: row.try_get("away_goals_extra_time")?,
        home_penalties: row.try_get("home_penalties")?,
        away_penalties: row.try_get("away_penalties")?,
        finalized_at: row.try_get("finalized_at")?,
        metadata: row.try_get("metadata")?,
    })
}

pub(super) fn match_event_from_row(
    row: &sqlx::postgres::PgRow,
) -> PersistenceResult<MatchReviewEventRecord> {
    let event_type = row
        .try_get::<String, _>("event_type")?
        .parse::<MatchEventType>()
        .map_err(PersistenceError::InvalidState)?;
    let verification_status = row
        .try_get::<String, _>("verification_status")?
        .parse::<MatchEventVerificationStatus>()
        .map_err(PersistenceError::InvalidState)?;
    let revision_status = row
        .try_get::<String, _>("revision_status")?
        .parse::<MatchEventRevisionStatus>()
        .map_err(PersistenceError::InvalidState)?;
    Ok(MatchReviewEventRecord {
        id: row.try_get("id")?,
        match_id: row.try_get("match_id")?,
        event_key: row.try_get("event_key")?,
        sequence_no: row.try_get("sequence_no")?,
        event_type,
        team_id: row.try_get("team_id")?,
        team_name: row.try_get("team_name")?,
        player_id: row.try_get("player_id")?,
        player_name: row.try_get("player_name")?,
        related_player_id: row.try_get("related_player_id")?,
        related_player_name: row.try_get("related_player_name")?,
        minute: row.try_get("minute")?,
        stoppage_minute: row.try_get("stoppage_minute")?,
        period: row.try_get("period")?,
        home_score: row.try_get("home_score")?,
        away_score: row.try_get("away_score")?,
        verification_status,
        revision_status,
        verified_at: row.try_get("verified_at")?,
        source_document_id: row.try_get("source_document_id")?,
        source_package_id: row.try_get("source_package_id")?,
        revision_of_event_id: row.try_get("revision_of_event_id")?,
        description: row.try_get("description")?,
        source_urls: row.try_get("source_urls")?,
        confidence: row.try_get("confidence")?,
        metadata: row.try_get("metadata")?,
        recorded_at: row.try_get("recorded_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}

pub(super) fn substitution_from_row(
    row: &sqlx::postgres::PgRow,
) -> PersistenceResult<SubstitutionRecord> {
    Ok(SubstitutionRecord {
        id: row.try_get("id")?,
        match_id: row.try_get("match_id")?,
        team_id: row.try_get("team_id")?,
        team_name: row.try_get("team_name")?,
        player_out_id: row.try_get("player_out_id")?,
        player_out_name: row.try_get("player_out_name")?,
        player_in_id: row.try_get("player_in_id")?,
        player_in_name: row.try_get("player_in_name")?,
        minute: row.try_get("minute")?,
        period: row.try_get("period")?,
        reason: row.try_get("reason")?,
    })
}
