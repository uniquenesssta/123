use super::aggregation::{aggregate_provider_rows, ProviderAggregate};
use uuid::Uuid;

fn close(actual: f64, expected: f64) {
    assert!((actual - expected).abs() < 1e-12, "{actual} != {expected}");
}

#[test]
fn empty_decisions_create_no_provider_groups() {
    assert!(aggregate_provider_rows(Vec::new()).unwrap().is_empty());
}

#[test]
fn empty_aggregate_keeps_zero_counts_means_and_score() {
    let aggregate = ProviderAggregate::default();
    assert_eq!(aggregate.total(), 0);
    assert_eq!(aggregate.scores(), (0.0, 0.0, 0.0, 0.0));
}

#[test]
fn unverifiable_only_keeps_timeliness_but_cannot_gain_weighted_accuracy() {
    let aggregate = ProviderAggregate {
        not_verifiable: 2,
        timeliness: vec![0.8, 1.0],
        reliability: vec![0.7],
        ..ProviderAggregate::default()
    };
    assert_eq!(aggregate.total(), 2);
    let (accuracy, timeliness, reliability, weighted) = aggregate.scores();
    assert_eq!(accuracy, 0.0);
    close(timeliness, 0.9);
    close(reliability, 0.7);
    assert_eq!(weighted, 0.0);
}

#[test]
fn mixed_decisions_use_separate_available_score_denominators() {
    let aggregate = ProviderAggregate {
        correct: 1,
        partial: 1,
        incorrect: 1,
        not_verifiable: 1,
        accuracy: vec![1.0, 0.5, 0.0],
        timeliness: vec![1.0, 0.0, 0.5, 0.5],
        reliability: vec![0.9, 0.4],
        ..ProviderAggregate::default()
    };
    assert_eq!(aggregate.total(), 4);
    let (accuracy, timeliness, reliability, weighted) = aggregate.scores();
    close(accuracy, 0.5);
    close(timeliness, 0.5);
    close(reliability, 0.65);
    close(weighted, 0.5375);
}

#[test]
fn weighted_score_uses_unrounded_means_before_original_six_digit_rounding() {
    let aggregate = ProviderAggregate {
        accuracy: vec![0.12345649],
        timeliness: vec![0.87654349],
        reliability: vec![0.23456749],
        ..ProviderAggregate::default()
    };
    let (accuracy, timeliness, reliability, weighted) = aggregate.scores();
    close(accuracy, 0.12345649);
    close(timeliness, 0.87654349);
    close(reliability, 0.23456749);
    close(weighted, 0.264198);
}

#[test]
fn missing_reliability_and_timeliness_keep_original_zero_defaults() {
    let aggregate = ProviderAggregate {
        correct: 2,
        accuracy: vec![1.0, 0.5],
        ..ProviderAggregate::default()
    };
    assert_eq!(aggregate.scores(), (0.75, 0.0, 0.0, 0.45));
}

#[test]
fn sample_count_uses_all_verdict_counts_without_score_vector_lengths() {
    let aggregate = ProviderAggregate {
        correct: 2,
        partial: 3,
        incorrect: 5,
        not_verifiable: 7,
        accuracy: vec![1.0],
        timeliness: vec![0.1, 0.2],
        ..ProviderAggregate::default()
    };
    assert_eq!(aggregate.total(), 17);
}

#[test]
fn snapshot_digest_preserves_decision_order_and_duplicate_fingerprints() {
    let mut aggregate = ProviderAggregate {
        decision_fingerprints: vec!["b".into(), "a".into(), "a".into()],
        ..ProviderAggregate::default()
    };
    let (digest, key) = aggregate.snapshot_identity(Uuid::nil(), "scope").unwrap();
    assert_eq!(
        digest,
        "58977f1fcf185e1f98dabcc8d070fcc1d71f785107cceeac8a01e645097d0ce8"
    );
    assert_eq!(key, "00000000-0000-0000-0000-000000000000:scope:58977f1fcf185e1f98dabcc8d070fcc1d71f785107cceeac8a01e645097d0ce8:postmatch-monitoring-v1");
    aggregate.decision_fingerprints.swap(0, 1);
    let (reordered, _) = aggregate.snapshot_identity(Uuid::nil(), "scope").unwrap();
    assert_eq!(
        reordered,
        "d6a4b4a908e6d9cf96911742ac9ae6b2f0d8726c050becef72bd263bd509a901"
    );
    aggregate.decision_fingerprints = vec!["b".into(), "a".into()];
    let (deduplicated, _) = aggregate.snapshot_identity(Uuid::nil(), "scope").unwrap();
    assert_eq!(
        deduplicated,
        "02d8bc3008a9bb0dcc4b86d7fd3428ced792355c733c19756bec5a56dc61b2c5"
    );
}

#[test]
fn snapshot_key_is_repeatable_and_separates_provider_and_partition() {
    let aggregate = ProviderAggregate {
        decision_fingerprints: vec!["x".into()],
        ..ProviderAggregate::default()
    };
    let provider = Uuid::from_u128(1);
    let first = aggregate
        .snapshot_identity(provider, "model:profile:parameter:T-1h")
        .unwrap();
    assert_eq!(
        first,
        aggregate
            .snapshot_identity(provider, "model:profile:parameter:T-1h")
            .unwrap()
    );
    let other_provider = aggregate
        .snapshot_identity(Uuid::from_u128(2), "model:profile:parameter:T-1h")
        .unwrap();
    let other_scope = aggregate
        .snapshot_identity(provider, "model:profile:parameter:T-6h")
        .unwrap();
    assert_eq!(first.0, other_provider.0);
    assert_eq!(first.0, other_scope.0);
    assert_ne!(first.1, other_provider.1);
    assert_ne!(first.1, other_scope.1);
}
