use super::input::{evidence_claim_fingerprint, prepared_conflict, validate_evidence_claim};
use super::parse_verification_state;
use crate::adapters::p4::idempotency::{ensure_idempotent_fingerprint, validate_idempotency_key};
use crate::{sha256_json, PersistenceError};
use chrono::{Duration, Utc};
use football_domain::{EvidenceClaimDraft, EvidenceConflictDraft, EvidenceVerificationState};
use serde_json::json;
use uuid::Uuid;

fn claim() -> EvidenceClaimDraft {
    let now = chrono::DateTime::parse_from_rfc3339("2026-10-08T12:00:00.123456789Z")
        .unwrap()
        .with_timezone(&Utc);
    EvidenceClaimDraft {
        match_id: Uuid::from_u128(1),
        entity_type: "team".into(),
        entity_id: Some(Uuid::from_u128(2)),
        field_key: "lineup".into(),
        value: json!({"status":"probable"}),
        verification_state: EvidenceVerificationState::Probable,
        source_tier: "official".into(),
        source_document_id: Some(Uuid::from_u128(3)),
        source_url: Some("https://example.test/a".into()),
        source_title: Some("source A".into()),
        source_domain: Some("example.test".into()),
        published_at: Some(now),
        observed_at: now,
        effective_at: Some(now),
        retrieved_at: now,
        timezone: "UTC".into(),
        independent_source_count: 1,
        conflict_group_id: None,
        research_run_id: Uuid::from_u128(4),
        prompt_version_id: None,
        prompt_version: None,
        schema_version_id: Uuid::from_u128(5),
        schema_version: "evidence-v1".into(),
        idempotency_key: "claim:fixed".into(),
        metadata: json!({"raw":"preserved"}),
    }
}

fn conflict() -> EvidenceConflictDraft {
    EvidenceConflictDraft {
        match_id: Uuid::from_u128(1),
        entity_type: "team".into(),
        entity_id: Some(Uuid::from_u128(2)),
        field_key: "lineup".into(),
        conflict_key: "conflict:fixed".into(),
        evidence_ids: vec![Uuid::from_u128(7), Uuid::from_u128(6), Uuid::from_u128(7)],
        trace_id: Uuid::from_u128(8),
        metadata: json!({"raw":"preserved"}),
    }
}

#[test]
fn source_states_and_complete_provenance_keep_original_policy() {
    for state in [
        EvidenceVerificationState::Confirmed,
        EvidenceVerificationState::Probable,
        EvidenceVerificationState::Conflict,
        EvidenceVerificationState::Stale,
        EvidenceVerificationState::NotFound,
        EvidenceVerificationState::NotApplicable,
    ] {
        let mut draft = claim();
        draft.verification_state = state;
        assert!(validate_evidence_claim(&draft).is_ok());
        for field in ["source_url", "source_title", "source_domain"] {
            for value in [serde_json::Value::Null, json!("")] {
                let mut raw = serde_json::to_value(&draft).unwrap();
                raw[field] = value;
                let changed = serde_json::from_value(raw).unwrap();
                assert_eq!(
                    validate_evidence_claim(&changed).is_err(),
                    state.requires_source(),
                    "{state:?}/{field}"
                );
            }
        }
        draft.source_url = Some(" ".into());
        draft.source_title = Some(" ".into());
        draft.source_domain = Some(" ".into());
        assert!(
            validate_evidence_claim(&draft).is_ok(),
            "existing policy checks empty strings without trimming source text"
        );
    }
}

#[test]
fn claim_mandatory_fields_keep_error_priority_and_raw_values() {
    for field in [
        "entity_type",
        "field_key",
        "source_tier",
        "timezone",
        "schema_version",
    ] {
        let mut raw = serde_json::to_value(claim()).unwrap();
        raw[field] = json!("  ");
        let changed = serde_json::from_value(raw).unwrap();
        assert!(
            matches!(validate_evidence_claim(&changed),Err(PersistenceError::InvalidState(m)) if m=="证据实体、字段、来源等级、时区和Schema版本不能为空"),
            "{field}"
        );
    }
    let mut draft = claim();
    draft.idempotency_key = " ".into();
    draft.entity_type = " ".into();
    draft.source_url = None;
    assert!(
        matches!(validate_evidence_claim(&draft),Err(PersistenceError::InvalidState(m)) if m=="幂等键不能为空且长度不得超过240")
    );
    let mut draft = claim();
    draft.field_key = " lineup ".into();
    validate_evidence_claim(&draft).unwrap();
    assert_eq!(draft.field_key, " lineup ");
}

#[test]
fn claim_retrieval_window_keeps_nanosecond_boundary() {
    let draft = claim();
    validate_evidence_claim(&draft).unwrap();
    let mut earlier = draft.clone();
    earlier.retrieved_at -= Duration::nanoseconds(1);
    assert!(
        matches!(validate_evidence_claim(&earlier),Err(PersistenceError::InvalidState(m)) if m=="retrieved_at不能早于observed_at")
    );
    let mut later = draft.clone();
    later.retrieved_at += Duration::nanoseconds(1);
    validate_evidence_claim(&later).unwrap();
    assert_ne!(
        evidence_claim_fingerprint(&draft, "value-hash").unwrap(),
        evidence_claim_fingerprint(&later, "value-hash").unwrap()
    );
}

#[test]
fn claim_fingerprint_preserves_semantic_identity_and_retry_key_policy() {
    let draft = claim();
    let content = sha256_json(&draft.value).unwrap();
    let original = evidence_claim_fingerprint(&draft, &content).unwrap();
    let other = Uuid::from_u128(99);
    for (field, value) in [
        ("match_id", json!(other)),
        ("entity_type", json!("player")),
        ("entity_id", json!(other)),
        ("field_key", json!("injury")),
        ("verification_state", json!("CONFIRMED")),
        ("source_tier", json!("secondary")),
        ("source_document_id", json!(other)),
        ("source_url", json!("https://example.test/b")),
        ("source_title", json!("source B")),
        ("source_domain", json!("other.test")),
        (
            "published_at",
            json!(draft.published_at.unwrap() + Duration::nanoseconds(1)),
        ),
        (
            "observed_at",
            json!(draft.observed_at + Duration::nanoseconds(1)),
        ),
        (
            "effective_at",
            json!(draft.effective_at.unwrap() + Duration::nanoseconds(1)),
        ),
        (
            "retrieved_at",
            json!(draft.retrieved_at + Duration::nanoseconds(1)),
        ),
        ("timezone", json!("Asia/Shanghai")),
        ("independent_source_count", json!(2)),
        ("conflict_group_id", json!(other)),
        ("research_run_id", json!(other)),
        ("prompt_version_id", json!(other)),
        ("prompt_version", json!("prompt-v2")),
        ("schema_version_id", json!(other)),
        ("schema_version", json!("evidence-v2")),
        ("metadata", json!({"raw":"changed"})),
    ] {
        let mut raw = serde_json::to_value(&draft).unwrap();
        raw[field] = value;
        let changed = serde_json::from_value(raw).unwrap();
        assert_ne!(
            original,
            evidence_claim_fingerprint(&changed, &content).unwrap(),
            "{field}"
        );
    }
    let mut changed = draft.clone();
    changed.idempotency_key = "another-key".into();
    assert_eq!(
        original,
        evidence_claim_fingerprint(&changed, &content).unwrap()
    );
    changed.value = json!({"status":"confirmed"});
    assert_ne!(
        original,
        evidence_claim_fingerprint(&changed, &sha256_json(&changed.value).unwrap()).unwrap()
    );
}

#[test]
fn conflict_preparation_deduplicates_sorts_and_keeps_identity_policy() {
    let draft = conflict();
    let (ids, fingerprint) = prepared_conflict(&draft).unwrap();
    assert_eq!(
        ids.into_iter().collect::<Vec<_>>(),
        vec![Uuid::from_u128(6), Uuid::from_u128(7)]
    );
    let mut reordered = draft.clone();
    reordered.evidence_ids = vec![Uuid::from_u128(6), Uuid::from_u128(7)];
    assert_eq!(fingerprint, prepared_conflict(&reordered).unwrap().1);
    reordered.conflict_key = "another-key".into();
    reordered.metadata = json!({"different":true});
    assert_eq!(
        fingerprint,
        prepared_conflict(&reordered).unwrap().1,
        "key/metadata are excluded by existing conflict fingerprint policy"
    );
    for (field, value) in [
        ("match_id", json!(Uuid::from_u128(99))),
        ("entity_type", json!("player")),
        ("entity_id", json!(null)),
        ("field_key", json!("injury")),
        ("trace_id", json!(Uuid::from_u128(99))),
        (
            "evidence_ids",
            json!([Uuid::from_u128(6), Uuid::from_u128(99)]),
        ),
    ] {
        let mut raw = serde_json::to_value(&draft).unwrap();
        raw[field] = value;
        assert_ne!(
            fingerprint,
            prepared_conflict(&serde_json::from_value(raw).unwrap())
                .unwrap()
                .1,
            "{field}"
        );
    }
}

#[test]
fn conflict_preflight_keeps_byte_key_limit_and_distinct_member_minimum() {
    for key in ["x".repeat(240), "中".repeat(80), " key ".into()] {
        validate_idempotency_key(&key).unwrap();
    }
    for key in [" ".into(), "x".repeat(241), "中".repeat(81)] {
        assert!(validate_idempotency_key(&key).is_err());
    }
    for ids in [
        vec![],
        vec![Uuid::from_u128(6)],
        vec![Uuid::from_u128(6); 2],
    ] {
        let mut draft = conflict();
        draft.evidence_ids = ids;
        assert!(
            matches!(prepared_conflict(&draft),Err(PersistenceError::InvalidState(m)) if m=="冲突组至少需要两条不同证据")
        );
    }
    let mut draft = conflict();
    draft.conflict_key = " ".into();
    draft.evidence_ids.clear();
    assert!(
        matches!(prepared_conflict(&draft),Err(PersistenceError::InvalidState(m)) if m=="幂等键不能为空且长度不得超过240")
    );
}

#[test]
fn verification_row_parser_keeps_six_states_and_unknown_errors() {
    for state in [
        EvidenceVerificationState::Confirmed,
        EvidenceVerificationState::Probable,
        EvidenceVerificationState::Conflict,
        EvidenceVerificationState::NotFound,
        EvidenceVerificationState::Stale,
        EvidenceVerificationState::NotApplicable,
    ] {
        assert_eq!(parse_verification_state(state.as_str()).unwrap(), state);
    }
    for value in ["confirmed", " CONFIRMED ", "UNKNOWN", ""] {
        assert!(
            matches!(parse_verification_state(value),Err(PersistenceError::InvalidState(m)) if m==format!("未知证据验证状态：{value}"))
        );
    }
}

#[test]
fn idempotent_retry_keeps_exact_fingerprint_and_original_error() {
    ensure_idempotent_fingerprint("证据声明", "key", "same", "same").unwrap();
    for (old, new) in [("same", "changed"), ("same", " same"), ("", "x")] {
        assert!(
            matches!(ensure_idempotent_fingerprint("证据冲突","key",old,new),Err(PersistenceError::InvalidState(m)) if m=="证据冲突幂等键key已绑定不同载荷")
        );
    }
}

#[test]
fn evidence_source_is_required_for_supported_facts() {
    let now = Utc::now();
    let claim = EvidenceClaimDraft {
        match_id: Uuid::new_v4(),
        entity_type: "team".to_string(),
        entity_id: None,
        field_key: "injury".to_string(),
        value: json!({"status": "out"}),
        verification_state: EvidenceVerificationState::Confirmed,
        source_tier: "official".to_string(),
        source_document_id: None,
        source_url: None,
        source_title: None,
        source_domain: None,
        published_at: Some(now),
        observed_at: now,
        effective_at: Some(now),
        retrieved_at: now,
        timezone: "UTC".to_string(),
        independent_source_count: 1,
        conflict_group_id: None,
        research_run_id: Uuid::new_v4(),
        prompt_version_id: None,
        prompt_version: None,
        schema_version_id: Uuid::new_v4(),
        schema_version: "evidence-v1".to_string(),
        idempotency_key: "evidence:test".to_string(),
        metadata: json!({}),
    };
    assert!(validate_evidence_claim(&claim).is_err());
}
