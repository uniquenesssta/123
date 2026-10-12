use crate::{PersistenceError, PersistenceResult};
use football_domain::{AbilityCandidateStatus, AbilityUpdateCandidateRecord};
use sqlx::Row;

pub(super) fn ability_candidate_from_row(
    row: &sqlx::postgres::PgRow,
) -> PersistenceResult<AbilityUpdateCandidateRecord> {
    let status: String = row.try_get("status")?;
    Ok(AbilityUpdateCandidateRecord {
        id: row.try_get("id")?,
        match_review_id: row.try_get("match_review_id")?,
        player_match_review_id: row.try_get("player_match_review_id")?,
        player_id: row.try_get("player_id")?,
        player_name: row.try_get("player_name")?,
        dimension_code: row.try_get("dimension_code")?,
        dimension_name: row.try_get("dimension_name")?,
        current_value: row.try_get("current_value")?,
        proposed_value: row.try_get("proposed_value")?,
        confidence: row.try_get("confidence")?,
        sample_size: row.try_get("sample_size")?,
        evidence: row.try_get("evidence")?,
        calculation_version: row.try_get("calculation_version")?,
        status: candidate_status_from_db(&status)?,
        created_at: row.try_get("created_at")?,
        decided_at: row.try_get("decided_at")?,
        decided_by: row.try_get("decided_by")?,
        decision_note: row.try_get("decision_note")?,
        accepted_observation_id: row.try_get("accepted_observation_id")?,
    })
}

pub(super) fn candidate_status_from_db(status: &str) -> PersistenceResult<AbilityCandidateStatus> {
    match status {
        "pending" => Ok(AbilityCandidateStatus::Pending),
        "accepted" => Ok(AbilityCandidateStatus::Accepted),
        "rejected" => Ok(AbilityCandidateStatus::Rejected),
        "superseded" => Ok(AbilityCandidateStatus::Superseded),
        other => Err(PersistenceError::InvalidState(format!(
            "未知候选状态：{other}"
        ))),
    }
}
