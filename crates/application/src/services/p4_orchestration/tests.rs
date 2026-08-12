use crate::use_cases::prediction::shared::p4_planning::{
    canonical_fact_keys, horizon_priority, is_p4_model,
};
use football_domain::P4Horizon;
use std::collections::BTreeSet;

#[test]
fn formal_fact_set_has_twenty_nine_unique_fields() {
    let fields = canonical_fact_keys();
    assert_eq!(fields.len(), 29);
    assert_eq!(fields.iter().collect::<BTreeSet<_>>().len(), 29);
}

#[test]
fn only_p4_models_enter_stage_f() {
    assert!(is_p4_model("p4"));
    assert!(is_p4_model("p4_knockout_90"));
    assert!(!is_p4_model("p7"));
}

#[test]
fn canonical_horizon_priority_increases_toward_kickoff() {
    assert!(horizon_priority(P4Horizon::T24h) < horizon_priority(P4Horizon::T6h));
    assert!(horizon_priority(P4Horizon::T6h) < horizon_priority(P4Horizon::T90m));
    assert!(horizon_priority(P4Horizon::T90m) < horizon_priority(P4Horizon::T1h));
}
