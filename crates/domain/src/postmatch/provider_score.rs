use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderScoreSnapshotRecord { pub id: Uuid, pub provider_id: Uuid, pub provider_name: String, pub scope_key: String, pub competition_id: Uuid, pub competition_profile_id: Uuid, pub model_version_id: Uuid, pub parameter_set_id: Uuid, pub horizon: String, pub sample_size: u64, pub correct_count: u64, pub partial_count: u64, pub incorrect_count: u64, pub not_verifiable_count: u64, pub accuracy_mean: f64, pub timeliness_mean: f64, pub reliability_mean: f64, pub weighted_score: f64, pub decision_set_sha256: String, pub calculation_version: String, pub generated_at: DateTime<Utc> }
