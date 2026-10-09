use crate::{PersistenceError, PersistenceResult};
use football_domain::{P4FreezeTaskEventRecord, P4FreezeTaskRecord, P4FreezeTaskState, P4Horizon};
use sqlx::Row;

pub(super) fn task_from_row(row: &sqlx::postgres::PgRow) -> PersistenceResult<P4FreezeTaskRecord> {
    Ok(P4FreezeTaskRecord {
        id: row.try_get("id")?,
        match_id: row.try_get("match_id")?,
        match_key: row.try_get("match_key")?,
        horizon: parse_horizon(row.try_get::<String, _>("horizon")?.as_str())?,
        kickoff_at: row.try_get("kickoff_at")?,
        data_cutoff_at: row.try_get("data_cutoff_at")?,
        research_due_at: row.try_get("research_due_at")?,
        freeze_deadline_at: row.try_get("freeze_deadline_at")?,
        rule_package_id: row.try_get("rule_package_id")?,
        model_version_id: row.try_get("model_version_id")?,
        parameter_set_id: row.try_get("parameter_set_id")?,
        competition_profile_id: row.try_get("competition_profile_id")?,
        research_schema_version_id: row.try_get("research_schema_version_id")?,
        snapshot_schema_version_id: row.try_get("snapshot_schema_version_id")?,
        requested_fact_keys: row.try_get("requested_fact_keys")?,
        trace_id: row.try_get("trace_id")?,
        state: parse_state(row.try_get::<String, _>("state")?.as_str())?,
        research_run_id: row.try_get("research_run_id")?,
        research_job_id: row.try_get("research_job_id")?,
        freeze_job_id: row.try_get("freeze_job_id")?,
        snapshot_id: row.try_get("snapshot_id")?,
        blockers: row.try_get("blockers")?,
        task_fingerprint: row.try_get("task_fingerprint")?,
        idempotency_key: row.try_get("idempotency_key")?,
        metadata: row.try_get("metadata")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}

pub(super) fn task_event_from_row(
    row: &sqlx::postgres::PgRow,
) -> PersistenceResult<P4FreezeTaskEventRecord> {
    let from_state = row
        .try_get::<Option<String>, _>("from_state")?
        .map(|value| parse_state(&value))
        .transpose()?;
    Ok(P4FreezeTaskEventRecord {
        id: row.try_get("id")?,
        task_id: row.try_get("task_id")?,
        from_state,
        to_state: parse_state(row.try_get::<String, _>("to_state")?.as_str())?,
        reason: row.try_get("reason")?,
        payload: row.try_get("payload")?,
        occurred_at: row.try_get("occurred_at")?,
    })
}

pub(super) fn parse_horizon(value: &str) -> PersistenceResult<P4Horizon> {
    match value {
        "T-24h" => Ok(P4Horizon::T24h),
        "T-6h" => Ok(P4Horizon::T6h),
        "T-90m" => Ok(P4Horizon::T90m),
        "T-1h" => Ok(P4Horizon::T1h),
        "T-N" => Ok(P4Horizon::LegacyTN),
        other => Err(PersistenceError::InvalidState(format!(
            "未知P4时点：{other}"
        ))),
    }
}

pub(super) fn parse_state(value: &str) -> PersistenceResult<P4FreezeTaskState> {
    P4FreezeTaskState::parse(value)
        .ok_or_else(|| PersistenceError::InvalidState(format!("未知P4冻结任务状态：{value}")))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn horizon_mapper_retains_legacy_read_compatibility_and_rejects_unknown_values() {
        for horizon in [
            P4Horizon::T24h,
            P4Horizon::T6h,
            P4Horizon::T90m,
            P4Horizon::T1h,
            P4Horizon::LegacyTN,
        ] {
            assert_eq!(parse_horizon(horizon.as_str()).unwrap(), horizon);
        }
        for bad in ["T-2h", "t-1h", " T-1h", "T-1h "] {
            assert!(parse_horizon(bad).is_err());
        }
    }
    #[test]
    fn state_mapper_requires_exact_persisted_state_values() {
        for state in P4FreezeTaskState::ALL {
            assert_eq!(parse_state(state.as_str()).unwrap(), state);
        }
        for bad in ["planned", " PLANNED", "UNKNOWN", ""] {
            assert!(parse_state(bad).is_err());
        }
    }
}
