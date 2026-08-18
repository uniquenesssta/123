use crate::{PersistenceError, PersistenceResult};
use std::collections::HashMap;
use uuid::Uuid;

pub(crate) fn calculate_probabilities(
    counts: &HashMap<Uuid, i32>,
    observed_matches: i32,
    alpha: f64,
) -> PersistenceResult<HashMap<Uuid, (f64, f64)>> {
    if counts.is_empty() {
        return Err(PersistenceError::InvalidState(
            "阵型概率计算至少需要一个阵型".to_string(),
        ));
    }
    if observed_matches == 0 {
        return Ok(counts
            .keys()
            .copied()
            .map(|formation_id| (formation_id, (1.0, 1.0)))
            .collect());
    }
    let formation_count = counts.len() as f64;
    let denominator = observed_matches as f64 + alpha;
    let prior = 1.0 / formation_count;
    Ok(counts
        .iter()
        .map(|(formation_id, usage_count)| {
            let raw = *usage_count as f64 / observed_matches as f64;
            let smoothed = (*usage_count as f64 + alpha * prior) / denominator;
            (*formation_id, (raw, smoothed))
        })
        .collect())
}
