use super::*;
use chrono::TimeZone;
use football_research_gateway::{ResearchSubject, ResearchValue, ResearchValueKind};

fn fact() -> ResearchFact {
    ResearchFact {
        fact_key: "home_injuries.player.1".to_string(),
        field_key: "home_injuries".to_string(),
        subject: ResearchSubject {
            entity_type: "player".to_string(),
            name: "Player A".to_string(),
            external_id: None,
        },
        value: ResearchValue {
            kind: ResearchValueKind::String,
            text: Some("injured".to_string()),
            number: None,
            integer: None,
            boolean: None,
            strings: vec![],
        },
        verification_state: "CONFIRMED".to_string(),
        source_urls: vec!["https://fifa.com/news/a".to_string()],
        published_at: Some(Utc.with_ymd_and_hms(2026, 7, 14, 8, 0, 0).unwrap()),
        observed_at: None,
        effective_at: None,
        timezone: Some("UTC".to_string()),
    }
}

#[test]
fn time_gate_rejects_future_and_missing_timezone() {
    let cutoff = Utc.with_ymd_and_hms(2026, 7, 14, 10, 0, 0).unwrap();
    let retrieved = Utc.with_ymd_and_hms(2026, 7, 14, 9, 59, 0).unwrap();
    let mut value = fact();
    value.published_at = Some(Utc.with_ymd_and_hms(2026, 7, 14, 11, 0, 0).unwrap());
    assert_eq!(
        audit_fact_time(&value, cutoff, retrieved).0,
        TimeAuditStatus::RejectedFuture
    );
    value.published_at = Some(Utc.with_ymd_and_hms(2026, 7, 14, 8, 0, 0).unwrap());
    value.timezone = None;
    assert_eq!(
        audit_fact_time(&value, cutoff, retrieved).0,
        TimeAuditStatus::RejectedMissingTimezone
    );
}

#[test]
fn entity_resolution_never_chooses_equal_top_candidates() {
    let left = EntityCandidate {
        entity_id: Uuid::new_v4(),
        canonical_name: "Player A".to_string(),
        matched_name: "Player A".to_string(),
        strategy: "alias".to_string(),
        score: 95,
        relation: Some("home".to_string()),
    };
    let right = EntityCandidate {
        entity_id: Uuid::new_v4(),
        canonical_name: "Player A 2".to_string(),
        matched_name: "Player A".to_string(),
        strategy: "alias".to_string(),
        score: 95,
        relation: Some("home".to_string()),
    };
    assert_eq!(
        decide_entity_resolution("player", &[left, right]).status,
        EntityResolutionStatus::Ambiguous
    );
}

#[test]
fn official_source_strictly_outranks_unclassified_conflict() {
    let official = RankedValue {
        key: "a".to_string(),
        value: json!("a"),
        evidence_ids: vec![Uuid::new_v4()],
        max_tier_rank: 500,
        independent_domains: 1,
        latest_evidence_at: None,
    };
    let unknown = RankedValue {
        key: "b".to_string(),
        value: json!("b"),
        evidence_ids: vec![Uuid::new_v4()],
        max_tier_rank: 100,
        independent_domains: 1,
        latest_evidence_at: None,
    };
    assert_eq!(
        conflict_winner(&[official.clone(), unknown])
            .expect("winner")
            .key,
        official.key
    );
}

#[test]
fn equal_rank_conflict_requires_manual_resolution() {
    let a = RankedValue {
        key: "a".to_string(),
        value: json!("a"),
        evidence_ids: vec![Uuid::new_v4()],
        max_tier_rank: 500,
        independent_domains: 1,
        latest_evidence_at: None,
    };
    let b = RankedValue {
        key: "b".to_string(),
        value: json!("b"),
        evidence_ids: vec![Uuid::new_v4()],
        max_tier_rank: 500,
        independent_domains: 1,
        latest_evidence_at: None,
    };
    assert!(conflict_winner(&[a, b]).is_none());
}

#[test]
fn route_registry_has_unique_model_entry_per_field() {
    validate_route_registry(&built_in_route_registry()).expect("route registry");
}

#[test]
fn source_policy_classifies_subdomains_without_guessing_unknown_domains() {
    let policy = built_in_source_policy().definition;
    let fifa = classify_source("inside.fifa.com", &policy).expect("source");
    assert_eq!(fifa.0, "official_competition");
    assert_eq!(fifa.2, "fifa.com");
    let unknown = classify_source("news.example", &policy).expect("source");
    assert_eq!(unknown.0, "unclassified");
    assert_eq!(unknown.2, "news.example");
}

#[test]
fn time_gate_rejects_results_retrieved_after_cutoff() {
    let cutoff = Utc.with_ymd_and_hms(2026, 7, 14, 10, 0, 0).unwrap();
    let retrieved = Utc.with_ymd_and_hms(2026, 7, 14, 10, 0, 1).unwrap();
    assert_eq!(
        audit_fact_time(&fact(), cutoff, retrieved).0,
        TimeAuditStatus::RejectedRetrievedAfterCutoff
    );
}

#[test]
fn source_url_rejects_insecure_or_embedded_credentials() {
    assert!(normalize_source_url("http://example.com/fact").is_err());
    assert!(normalize_source_url("https://user:secret@example.com/fact").is_err());
    assert!(normalize_source_url("https://example.com/fact#section").is_ok());
}

#[test]
fn source_tier_is_derived_from_url_domain_not_provider_label() {
    let command = ProcessResearchEvidenceCommand {
        research_run_id: Uuid::new_v4(),
        response_id: "resp_1".to_string(),
        retrieved_at: Utc.with_ymd_and_hms(2026, 7, 14, 9, 0, 0).unwrap(),
        output: ResearchOutput {
            schema_version: football_domain::P4_RESEARCH_OUTPUT_SCHEMA_VERSION.to_string(),
            match_key: "match-1".to_string(),
            data_cutoff_at: Utc.with_ymd_and_hms(2026, 7, 14, 10, 0, 0).unwrap(),
            facts: vec![],
            missing_fields: vec![],
        },
        citations: vec![],
        sources: vec![WebSource {
            url: "https://news.example/fact".to_string(),
            title: Some("Fact".to_string()),
            domain: "fifa.com".to_string(),
        }],
    };
    let index =
        build_source_index(&command, &built_in_source_policy().definition).expect("source index");
    let source = index.values().next().expect("source");
    assert_eq!(source.domain, "news.example");
    assert_eq!(source.tier, "unclassified");
}
