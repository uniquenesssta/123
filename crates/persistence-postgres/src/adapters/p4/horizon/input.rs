use crate::{sha256_json, PersistenceError, PersistenceResult};
use football_domain::P4FreezeTaskDraft;
use serde_json::json;

pub(super) struct PreparedTask {
    pub(super) requested_fact_keys: Vec<String>,
    pub(super) task_fingerprint: String,
}

pub(super) fn prepare(draft: &P4FreezeTaskDraft) -> PersistenceResult<PreparedTask> {
    if !draft.horizon.is_canonical() {
        return Err(PersistenceError::InvalidState(
            "T-N兼容时点不得进入P4正式冻结队列".to_string(),
        ));
    }
    if draft.requested_fact_keys.is_empty() {
        return Err(PersistenceError::InvalidState(
            "P4冻结任务至少需要一个事实字段".to_string(),
        ));
    }
    let mut requested_fact_keys = draft.requested_fact_keys.clone();
    requested_fact_keys.sort();
    requested_fact_keys.dedup();
    let task_fingerprint = sha256_json(&json!({
        "match_id": draft.match_id,
        "match_key": draft.match_key,
        "horizon": draft.horizon.as_str(),
        "kickoff_at": draft.kickoff_at,
        "data_cutoff_at": draft.data_cutoff_at,
        "research_due_at": draft.research_due_at,
        "freeze_deadline_at": draft.freeze_deadline_at,
        "rule_package_id": draft.rule_package_id,
        "model_version_id": draft.model_version_id,
        "parameter_set_id": draft.parameter_set_id,
        "competition_profile_id": draft.competition_profile_id,
        "research_schema_version_id": draft.research_schema_version_id,
        "snapshot_schema_version_id": draft.snapshot_schema_version_id,
        "requested_fact_keys": &requested_fact_keys,
        "trace_id": draft.trace_id,
        "state": draft.state.as_str(),
        "metadata": &draft.metadata,
    }))?;
    Ok(PreparedTask {
        requested_fact_keys,
        task_fingerprint,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Duration, Utc};
    use football_domain::{P4FreezeTaskState, P4Horizon};
    use uuid::Uuid;
    fn draft() -> P4FreezeTaskDraft {
        let now = Utc::now();
        P4FreezeTaskDraft {
            match_id: Uuid::from_u128(1),
            match_key: "match".into(),
            horizon: P4Horizon::T1h,
            kickoff_at: now + Duration::hours(1),
            data_cutoff_at: now,
            research_due_at: now - Duration::minutes(15),
            freeze_deadline_at: now + Duration::minutes(15),
            rule_package_id: Uuid::from_u128(2),
            model_version_id: Uuid::from_u128(3),
            parameter_set_id: Uuid::from_u128(4),
            competition_profile_id: Uuid::from_u128(5),
            research_schema_version_id: Uuid::from_u128(6),
            snapshot_schema_version_id: Uuid::from_u128(7),
            requested_fact_keys: vec!["b".into(), "a".into(), "a".into()],
            trace_id: Uuid::from_u128(8),
            state: P4FreezeTaskState::Planned,
            idempotency_key: "task".into(),
            metadata: json!({"version":1}),
        }
    }
    #[test]
    fn canonical_task_preflight_rejects_compatibility_horizons_and_empty_facts() {
        for horizon in [P4Horizon::T90m, P4Horizon::LegacyTN] {
            let mut input = draft();
            input.horizon = horizon;
            assert!(prepare(&input).is_err());
        }
        let mut input = draft();
        input.requested_fact_keys.clear();
        assert!(prepare(&input).is_err());
        for horizon in P4Horizon::CANONICAL {
            let mut input = draft();
            input.horizon = horizon;
            assert!(prepare(&input).is_ok());
        }
    }
    #[test]
    fn task_fingerprint_normalizes_only_fact_order_and_duplicates() {
        let input = draft();
        let prepared = prepare(&input).unwrap();
        assert_eq!(prepared.requested_fact_keys, ["a", "b"]);
        let mut retry = input.clone();
        retry.requested_fact_keys = vec!["a".into(), "b".into()];
        assert_eq!(
            prepared.task_fingerprint,
            prepare(&retry).unwrap().task_fingerprint
        );
        for field in [
            "trace_id",
            "metadata",
            "state",
            "research_schema_version_id",
            "snapshot_schema_version_id",
            "requested_fact_keys",
        ] {
            let mut changed = serde_json::to_value(&input).unwrap();
            changed[field] = match field {
                "metadata" => json!({"version":2}),
                "state" => json!("MISSED"),
                "requested_fact_keys" => json!(["different"]),
                _ => json!(Uuid::from_u128(999)),
            };
            let changed: P4FreezeTaskDraft = serde_json::from_value(changed).unwrap();
            assert_ne!(
                prepared.task_fingerprint,
                prepare(&changed).unwrap().task_fingerprint,
                "{field}"
            );
        }
    }
    #[test]
    fn task_fingerprint_preserves_submicrosecond_time_identity() {
        let input = draft();
        let mut changed = input.clone();
        changed.data_cutoff_at += Duration::nanoseconds(1);
        assert_ne!(
            prepare(&input).unwrap().task_fingerprint,
            prepare(&changed).unwrap().task_fingerprint
        );
    }
}
