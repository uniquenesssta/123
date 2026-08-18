use super::{probability::calculate_probabilities, validation::validate_distribution_draft};
use crate::adapters::catalog::formations::constants::UNKNOWN_FORMATION_ID;
use chrono::NaiveDate;
use football_domain::{FormationUsageDistributionDraft, FormationUsageEntryDraft};
use serde_json::json;
use std::collections::HashMap;
use uuid::Uuid;

fn draft(scope: &str) -> FormationUsageDistributionDraft {
    FormationUsageDistributionDraft {
        scope_type: scope.into(),
        team_id: Some(Uuid::new_v4()),
        coach_id: None,
        competition_id: None,
        window_preset: "custom".into(),
        window_start: NaiveDate::from_ymd_opt(2026, 1, 1),
        window_end: NaiveDate::from_ymd_opt(2026, 1, 31),
        observed_matches: 10,
        confidence: 0.8,
        alpha: 3.0,
        source_document_id: None,
        metadata: json!({}),
        entries: vec![FormationUsageEntryDraft {
            formation_id: Uuid::new_v4(),
            usage_count: 7,
        }],
    }
}

#[test]
fn scope_shape_and_bounds_stay_strict() {
    assert!(validate_distribution_draft(&draft("team")).is_ok());
    let mut invalid = draft("coach");
    invalid.coach_id = None;
    assert!(validate_distribution_draft(&invalid).is_err());
    let mut bounded = draft("team");
    bounded.confidence = 1.1;
    assert!(validate_distribution_draft(&bounded).is_err());
    bounded.confidence = 0.8;
    bounded.alpha = 0.0;
    assert!(validate_distribution_draft(&bounded).is_err());
}

#[test]
fn smoothed_probabilities_remain_normalized() {
    let counts = HashMap::from([
        (Uuid::new_v4(), 6),
        (Uuid::new_v4(), 3),
        (UNKNOWN_FORMATION_ID, 1),
    ]);
    let p = calculate_probabilities(&counts, 10, 3.0).expect("probabilities");
    assert!((p.values().map(|v| v.0).sum::<f64>() - 1.0).abs() < 1e-9);
    assert!((p.values().map(|v| v.1).sum::<f64>() - 1.0).abs() < 1e-9);
}
