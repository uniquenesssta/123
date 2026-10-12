use crate::{postmatch::round6, sha256_json, PersistenceResult};
use football_domain::POSTMATCH_MONITORING_VERSION;
use sqlx::Row;
use std::collections::BTreeMap;
use uuid::Uuid;

#[derive(Debug, Default)]
pub(super) struct ProviderAggregate {
    pub(super) correct: usize,
    pub(super) partial: usize,
    pub(super) incorrect: usize,
    pub(super) not_verifiable: usize,
    pub(super) accuracy: Vec<f64>,
    pub(super) timeliness: Vec<f64>,
    pub(super) reliability: Vec<f64>,
    pub(super) decision_fingerprints: Vec<String>,
}

impl ProviderAggregate {
    pub(super) fn total(&self) -> usize {
        self.correct + self.partial + self.incorrect + self.not_verifiable
    }

    pub(super) fn scores(&self) -> (f64, f64, f64, f64) {
        let accuracy_mean = mean(&self.accuracy).unwrap_or(0.0);
        let timeliness_mean = mean(&self.timeliness).unwrap_or(0.0);
        let reliability_mean = mean(&self.reliability).unwrap_or(0.0);
        let weighted_score = if self.accuracy.is_empty() {
            0.0
        } else {
            round6(accuracy_mean * 0.60 + timeliness_mean * 0.15 + reliability_mean * 0.25)
        };
        (
            accuracy_mean,
            timeliness_mean,
            reliability_mean,
            weighted_score,
        )
    }

    pub(super) fn snapshot_identity(
        &self,
        provider_id: Uuid,
        partition_key: &str,
    ) -> PersistenceResult<(String, String)> {
        let decision_set_sha256 = sha256_json(&self.decision_fingerprints)?;
        let snapshot_key = format!(
            "{}:{}:{}:{}",
            provider_id, partition_key, decision_set_sha256, POSTMATCH_MONITORING_VERSION
        );
        Ok((decision_set_sha256, snapshot_key))
    }
}

pub(super) fn aggregate_provider_rows(
    rows: Vec<sqlx::postgres::PgRow>,
) -> PersistenceResult<BTreeMap<Uuid, ProviderAggregate>> {
    let mut groups: BTreeMap<Uuid, ProviderAggregate> = BTreeMap::new();
    for row in rows {
        let provider_id: Uuid = row.try_get("provider_id")?;
        let entry = groups.entry(provider_id).or_default();
        let verdict: String = row.try_get("verdict")?;
        match verdict.as_str() {
            "correct" => entry.correct += 1,
            "partial" => entry.partial += 1,
            "incorrect" => entry.incorrect += 1,
            _ => entry.not_verifiable += 1,
        }
        if let Some(value) = row.try_get::<Option<f64>, _>("accuracy_score")? {
            entry.accuracy.push(value);
        }
        if let Some(value) = row.try_get::<Option<f64>, _>("reliability_score")? {
            entry.reliability.push(value);
        }
        entry.timeliness.push(row.try_get("timeliness_score")?);
        entry
            .decision_fingerprints
            .push(row.try_get("decision_fingerprint")?);
    }
    Ok(groups)
}

fn mean(values: &[f64]) -> Option<f64> {
    (!values.is_empty()).then(|| values.iter().sum::<f64>() / values.len() as f64)
}
