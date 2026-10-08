use super::command::ProcessResearchEvidenceCommand;
use super::conflict::conflict_winner;
use super::entity_resolution::decide_entity_resolution;
use super::process::process_p4_research_evidence;
use super::routing::{built_in_route_registry, validate_route_registry};
use super::source_policy::{
    build_source_index, built_in_source_policy, classify_source, normalize_source_url,
};
use super::time_audit::audit_fact_time;
use super::types::RankedValue;
use crate::ports::research::{
    FactPipelinePort, ResearchEvidenceLedgerPort, SerializedConflictEventPayload,
};
use crate::ports::{PortError, PortErrorKind, PortResult};
use crate::ApplicationError;
use async_trait::async_trait;
use chrono::Utc;
use football_domain::{
    ConflictEvaluationDraft, ConflictEvaluationRecord, EntityCandidate, EntityResolutionDraft,
    EntityResolutionRecord, EntityResolutionStatus, EvidenceClaimDraft, EvidenceClaimRecord,
    EvidenceConflictDraft, EvidenceConflictRecord, EvidenceRouteDraft, EvidenceRouteRecord,
    FactPipelineContext, TimeAuditDraft, TimeAuditRecord, TimeAuditStatus,
};
use football_research_gateway::{ResearchFact, ResearchOutput, WebSource};
use serde_json::json;
use serde_json::Value;
use std::sync::Mutex;
use uuid::Uuid;

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

struct PipelineProbe {
    context: FactPipelineContext,
    candidates: Vec<EntityCandidate>,
    calls: Mutex<Vec<(&'static str, Value)>>,
    failure: Option<(&'static str, PortError)>,
}

impl PipelineProbe {
    fn new() -> Self {
        Self {
            context: FactPipelineContext {
                research_run_id: Uuid::new_v4(),
                match_id: Uuid::new_v4(),
                match_key: "match-1".into(),
                horizon: "T-24h".into(),
                data_cutoff_at: Utc.with_ymd_and_hms(2026, 7, 14, 10, 0, 0).unwrap(),
                trace_id: Uuid::new_v4(),
                prompt_version_id: None,
                prompt_version: None,
                schema_version_id: Uuid::new_v4(),
                schema_version: football_domain::P4_RESEARCH_OUTPUT_SCHEMA_VERSION.into(),
                home_team_id: Some(Uuid::new_v4()),
                home_team_name: Some("Home".into()),
                away_team_id: Some(Uuid::new_v4()),
                away_team_name: Some("Away".into()),
                competition_id: None,
                competition_name: None,
                competition_code: None,
            },
            candidates: vec![EntityCandidate {
                entity_id: Uuid::new_v4(),
                canonical_name: "Player A".into(),
                matched_name: "Player A".into(),
                strategy: "alias".into(),
                score: 95,
                relation: Some("home".into()),
            }],
            calls: Mutex::new(vec![]),
            failure: None,
        }
    }

    fn command(&self) -> ProcessResearchEvidenceCommand {
        ProcessResearchEvidenceCommand {
            research_run_id: self.context.research_run_id,
            response_id: "resp_1".into(),
            retrieved_at: self.context.data_cutoff_at - chrono::Duration::minutes(1),
            output: ResearchOutput {
                schema_version: self.context.schema_version.clone(),
                match_key: self.context.match_key.clone(),
                data_cutoff_at: self.context.data_cutoff_at,
                facts: vec![fact()],
                missing_fields: vec![],
            },
            citations: vec![],
            sources: vec![WebSource {
                url: "https://fifa.com/news/a".into(),
                title: Some("Fact".into()),
                domain: "fifa.com".into(),
            }],
        }
    }

    fn conflict_command(&self) -> ProcessResearchEvidenceCommand {
        let mut command = self.command();
        let mut second = fact();
        second.fact_key = "home_injuries.player.2".into();
        second.value.text = Some("fit".into());
        second.source_urls = vec!["https://news.example/fact".into()];
        command.output.facts.push(second);
        command.sources.push(WebSource {
            url: "https://news.example/fact".into(),
            title: None,
            domain: "news.example".into(),
        });
        command
    }

    fn record(&self, step: &'static str, value: Value) -> PortResult<()> {
        self.calls.lock().unwrap().push((step, value));
        if let Some((failed_step, error)) = &self.failure {
            if *failed_step == step {
                return Err(error.clone());
            }
        }
        Ok(())
    }

    fn steps(&self) -> Vec<&'static str> {
        self.calls
            .lock()
            .unwrap()
            .iter()
            .map(|(step, _)| *step)
            .collect()
    }

    fn drafts(&self, step: &str) -> Vec<Value> {
        self.calls
            .lock()
            .unwrap()
            .iter()
            .filter(|(name, _)| *name == step)
            .map(|(_, value)| value.clone())
            .collect()
    }
}

#[async_trait]
impl FactPipelinePort for PipelineProbe {
    async fn context(&self, research_run_id: Uuid) -> PortResult<FactPipelineContext> {
        assert_eq!(research_run_id, self.context.research_run_id);
        self.record("context", json!(research_run_id))?;
        Ok(self.context.clone())
    }
    async fn find_entity_candidates(
        &self,
        context: &FactPipelineContext,
        entity_type: &str,
        normalized_name: &str,
        compact_name: &str,
        external_id: Option<&str>,
    ) -> PortResult<Vec<EntityCandidate>> {
        self.record(
            "candidates",
            json!({"research_run_id":context.research_run_id,
            "entity_type":entity_type,"normalized_name":normalized_name,
            "compact_name":compact_name,"external_id":external_id}),
        )?;
        Ok(self.candidates.clone())
    }
    async fn append_entity_resolution(
        &self,
        draft: &EntityResolutionDraft,
    ) -> PortResult<EntityResolutionRecord> {
        self.record("resolution", json!(draft))?;
        Ok(EntityResolutionRecord {
            id: Uuid::new_v4(),
            status: draft.status,
            resolved_entity_id: draft.resolved_entity_id,
            resolution_fingerprint: "resolution".into(),
            created_at: Utc::now(),
        })
    }
    async fn append_time_audit(&self, draft: &TimeAuditDraft) -> PortResult<TimeAuditRecord> {
        self.record("time", json!(draft))?;
        Ok(TimeAuditRecord {
            id: Uuid::new_v4(),
            status: draft.status,
            time_fingerprint: "time".into(),
            created_at: Utc::now(),
        })
    }
    async fn append_conflict_evaluation(
        &self,
        draft: &ConflictEvaluationDraft,
    ) -> PortResult<ConflictEvaluationRecord> {
        self.record("evaluation", json!(draft))?;
        Ok(ConflictEvaluationRecord {
            id: Uuid::new_v4(),
            status: draft.status,
            evaluation_fingerprint: "evaluation".into(),
            created_at: Utc::now(),
        })
    }
    async fn append_conflict_event(
        &self,
        conflict_id: Uuid,
        event_type: &str,
        actor: &str,
        payload: &SerializedConflictEventPayload,
        idempotency_key: &str,
    ) -> PortResult<()> {
        self.record(
            "event",
            json!({"conflict_id":conflict_id,"event_type":event_type,
            "actor":actor,"payload":payload.0,"idempotency_key":idempotency_key}),
        )
    }
    async fn append_evidence_route(
        &self,
        draft: &EvidenceRouteDraft,
    ) -> PortResult<EvidenceRouteRecord> {
        self.record("route", json!(draft))?;
        Ok(EvidenceRouteRecord {
            id: Uuid::new_v4(),
            status: draft.status,
            route_fingerprint: "route".into(),
            created_at: Utc::now(),
        })
    }
}

#[async_trait]
impl ResearchEvidenceLedgerPort for PipelineProbe {
    async fn append_evidence_claim(
        &self,
        draft: &EvidenceClaimDraft,
    ) -> PortResult<EvidenceClaimRecord> {
        self.record("claim", json!(draft))?;
        Ok(EvidenceClaimRecord {
            id: Uuid::new_v4(),
            match_id: draft.match_id,
            field_key: draft.field_key.clone(),
            verification_state: draft.verification_state,
            content_sha256: "content".into(),
            claim_fingerprint: "claim".into(),
            idempotency_key: draft.idempotency_key.clone(),
            created_at: Utc::now(),
        })
    }
    async fn create_evidence_conflict(
        &self,
        draft: &EvidenceConflictDraft,
    ) -> PortResult<EvidenceConflictRecord> {
        self.record("conflict", json!(draft))?;
        Ok(EvidenceConflictRecord {
            id: Uuid::new_v4(),
            conflict_key: draft.conflict_key.clone(),
            created_at: Utc::now(),
        })
    }
}

#[tokio::test]
async fn invalid_command_and_context_stop_before_ledger_writes() {
    let probe = PipelineProbe::new();
    let mut command = probe.command();
    command.response_id = " ".into();
    assert!(matches!(
        process_p4_research_evidence(&probe, command).await,
        Err(ApplicationError::Validation(_))
    ));
    assert!(probe.steps().is_empty());
    for field in ["match", "cutoff", "schema"] {
        let probe = PipelineProbe::new();
        let mut command = probe.command();
        match field {
            "match" => command.output.match_key = "other-match".into(),
            "cutoff" => command.output.data_cutoff_at += chrono::Duration::nanoseconds(1),
            "schema" => command.output.schema_version = "other-schema".into(),
            _ => unreachable!(),
        }
        assert!(matches!(
            process_p4_research_evidence(&probe, command).await,
            Err(ApplicationError::Validation(_))
        ));
        assert_eq!(probe.steps(), ["context"]);
    }
}

#[tokio::test]
async fn pipeline_preserves_context_source_and_repeatable_idempotency_keys() {
    let probe = PipelineProbe::new();
    let summary = process_p4_research_evidence(&probe, probe.command())
        .await
        .unwrap();
    assert_eq!(
        probe.steps(),
        [
            "context",
            "candidates",
            "resolution",
            "time",
            "claim",
            "route"
        ]
    );
    assert_eq!(
        (
            summary.fact_count,
            summary.resolved_entity_count,
            summary.evidence_claim_count,
            summary.routed_count,
            summary.blocked_count
        ),
        (1, 1, 1, 1, 0)
    );
    let claim = probe.drafts("claim")[0].clone();
    assert_eq!(claim["match_id"], json!(probe.context.match_id));
    assert_eq!(
        claim["research_run_id"],
        json!(probe.context.research_run_id)
    );
    assert_eq!(
        claim["schema_version_id"],
        json!(probe.context.schema_version_id)
    );
    assert_eq!(claim["entity_id"], json!(probe.candidates[0].entity_id));
    assert_eq!(claim["source_domain"], "fifa.com");
    assert_eq!(claim["source_tier"], "official_competition");
    assert_eq!(claim["verification_state"], "CONFIRMED");
    assert_eq!(
        probe.drafts("time")[0]["data_cutoff_at"],
        json!(probe.context.data_cutoff_at)
    );
    assert_eq!(probe.drafts("route")[0]["selected_value"], claim["value"]);
    process_p4_research_evidence(&probe, probe.command())
        .await
        .unwrap();
    for step in ["resolution", "time", "claim", "route"] {
        let drafts = probe.drafts(step);
        assert_eq!(
            drafts[0]["idempotency_key"], drafts[1]["idempotency_key"],
            "{step}"
        );
    }
}

#[tokio::test]
async fn opposite_team_candidate_never_enters_home_route() {
    let mut probe = PipelineProbe::new();
    probe.candidates[0].relation = Some("away".into());
    let summary = process_p4_research_evidence(&probe, probe.command())
        .await
        .unwrap();
    assert_eq!(
        (
            summary.unmatched_entity_count,
            summary.routed_count,
            summary.blocked_count
        ),
        (1, 0, 1)
    );
    assert_eq!(probe.drafts("resolution")[0]["status"], "unmatched");
    assert_eq!(probe.drafts("route")[0]["status"], "blocked_entity");
    assert!(probe.drafts("claim")[0]["entity_id"].is_null());
}

#[tokio::test]
async fn missing_source_preserves_prior_resolution_and_time_audit_only() {
    let probe = PipelineProbe::new();
    let mut command = probe.command();
    command.sources.clear();
    let error = process_p4_research_evidence(&probe, command)
        .await
        .unwrap_err();
    assert!(
        matches!(error, ApplicationError::Validation(message) if message.contains("来源没有出现在已验证引用索引中"))
    );
    assert_eq!(
        probe.steps(),
        ["context", "candidates", "resolution", "time"]
    );
}

#[tokio::test]
async fn port_errors_keep_kind_message_and_stop_at_the_failed_step() {
    let steps = [
        "context",
        "candidates",
        "resolution",
        "time",
        "claim",
        "route",
    ];
    for (index, kind) in [
        PortErrorKind::Unavailable,
        PortErrorKind::NotFound,
        PortErrorKind::Conflict,
        PortErrorKind::InvalidState,
        PortErrorKind::Serialization,
        PortErrorKind::Infrastructure,
    ]
    .into_iter()
    .enumerate()
    {
        let mut probe = PipelineProbe::new();
        let expected = PortError::new(kind, format!("failed:{}", steps[index]));
        probe.failure = Some((steps[index], expected.clone()));
        let error = process_p4_research_evidence(&probe, probe.command())
            .await
            .unwrap_err();
        assert!(matches!(error, ApplicationError::Port(actual) if actual == expected));
        assert_eq!(probe.steps(), steps[..=index]);
    }
    let conflict_steps = [
        "context",
        "candidates",
        "resolution",
        "time",
        "candidates",
        "resolution",
        "time",
        "claim",
        "claim",
        "conflict",
        "evaluation",
        "event",
        "route",
    ];
    for step in ["conflict", "evaluation", "event", "route"] {
        let mut probe = PipelineProbe::new();
        let expected = PortError::new(PortErrorKind::Conflict, format!("failed:{step}"));
        probe.failure = Some((step, expected.clone()));
        let error = process_p4_research_evidence(&probe, probe.conflict_command())
            .await
            .unwrap_err();
        assert!(matches!(error, ApplicationError::Port(actual) if actual == expected));
        let end = conflict_steps
            .iter()
            .position(|current| *current == step)
            .unwrap();
        assert_eq!(probe.steps(), conflict_steps[..=end]);
    }
}

#[tokio::test]
async fn missing_field_retrieved_after_cutoff_is_stale_and_blocked() {
    let probe = PipelineProbe::new();
    let mut command = probe.command();
    command.output.facts.clear();
    command
        .output
        .missing_fields
        .push(football_research_gateway::MissingField {
            field_key: "home_injuries".into(),
            verification_state: "NOT_FOUND".into(),
        });
    command.retrieved_at = probe.context.data_cutoff_at + chrono::Duration::nanoseconds(1);
    let summary = process_p4_research_evidence(&probe, command).await.unwrap();
    assert_eq!(probe.steps(), ["context", "time", "claim", "route"]);
    assert_eq!(
        (
            summary.missing_field_count,
            summary.time_rejected_count,
            summary.evidence_claim_count,
            summary.blocked_count
        ),
        (1, 1, 1, 1)
    );
    assert_eq!(
        probe.drafts("time")[0]["status"],
        "rejected_retrieved_after_cutoff"
    );
    assert_eq!(probe.drafts("claim")[0]["verification_state"], "STALE");
    assert_eq!(probe.drafts("route")[0]["status"], "blocked_time");
    assert!(probe.drafts("route")[0]["selected_value"].is_null());
}

#[tokio::test]
async fn official_conflict_resolution_writes_evaluation_event_then_selected_route() {
    let probe = PipelineProbe::new();
    let command = probe.conflict_command();
    let summary = process_p4_research_evidence(&probe, command).await.unwrap();
    assert_eq!(
        probe.steps(),
        [
            "context",
            "candidates",
            "resolution",
            "time",
            "candidates",
            "resolution",
            "time",
            "claim",
            "claim",
            "conflict",
            "evaluation",
            "event",
            "route"
        ]
    );
    assert_eq!(
        (
            summary.conflict_count,
            summary.auto_resolved_conflict_count,
            summary.manual_conflict_count,
            summary.routed_count
        ),
        (1, 1, 0, 1)
    );
    assert_eq!(probe.drafts("evaluation")[0]["status"], "auto_resolved");
    assert_eq!(probe.drafts("event")[0]["event_type"], "resolved");
    assert_eq!(
        probe.drafts("event")[0]["actor"],
        "deterministic-conflict-resolver"
    );
    assert_eq!(
        probe.drafts("route")[0]["selected_value"]["text"],
        "injured"
    );
    assert_eq!(
        probe.drafts("route")[0]["selected_evidence_ids"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
}
