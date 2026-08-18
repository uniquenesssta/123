use crate::{
    adapters::catalog::formations::constants::UNKNOWN_FORMATION_ID, PersistenceError,
    PersistenceResult,
};
use football_domain::FormationUsageDistributionDraft;
use std::collections::HashMap;
use uuid::Uuid;

pub(super) fn collect_usage_counts(
    draft: &FormationUsageDistributionDraft,
) -> PersistenceResult<HashMap<Uuid, i32>> {
    let mut counts = HashMap::new();
    for entry in &draft.entries {
        if entry.usage_count < 0 || entry.usage_count > draft.observed_matches {
            return Err(PersistenceError::InvalidState(
                "阵型使用次数必须位于 0 到观察场数之间".to_string(),
            ));
        }
        if counts
            .insert(entry.formation_id, entry.usage_count)
            .is_some()
        {
            return Err(PersistenceError::InvalidState(
                "同一阵型在一个观察窗口中不能重复".to_string(),
            ));
        }
    }
    Ok(counts)
}

pub(super) fn complete_usage_counts(
    mut counts: HashMap<Uuid, i32>,
    observed_matches: i32,
) -> PersistenceResult<HashMap<Uuid, i32>> {
    let total: i32 = counts.values().sum();
    if total > observed_matches {
        return Err(PersistenceError::InvalidState(
            "阵型使用次数合计不能超过观察场数".to_string(),
        ));
    }
    let missing = observed_matches - total;
    if observed_matches == 0 {
        counts.clear();
        counts.insert(UNKNOWN_FORMATION_ID, 0);
    } else if missing > 0 {
        *counts.entry(UNKNOWN_FORMATION_ID).or_insert(0) += missing;
    }
    counts.retain(|formation_id, count| *count > 0 || *formation_id == UNKNOWN_FORMATION_ID);
    Ok(counts)
}
