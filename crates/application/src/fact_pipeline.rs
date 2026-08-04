use super::{ApplicationError, ApplicationResult, ApplicationService};
use chrono::{DateTime, Utc};
use football_domain::{
    ConflictEvaluationDraft, ConflictEvaluationStatus, EntityCandidate, EntityResolutionDraft,
    EntityResolutionRecord, EntityResolutionStatus, EvidenceClaimDraft, EvidenceClaimRecord,
    EvidenceConflictDraft, EvidenceRouteDraft, EvidenceRouteRegistry, EvidenceRouteRule,
    EvidenceRouteStatus, EvidenceVerificationState, FactPipelineContext, FactPipelineSummary,
    SourcePolicyDefinition, SourcePolicyVersionDraft, TimeAuditDraft, TimeAuditRecord,
    TimeAuditStatus, P4_EVIDENCE_ROUTE_VERSION, P4_SOURCE_POLICY_VERSION,
};
use football_research_gateway::{
    MissingField, ResearchFact, ResearchOutput, WebCitation, WebSource,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};
use url::Url;
use uuid::Uuid;

const SOURCE_POLICY_KEY: &str = "p4-default-source-policy";
const SOURCE_POLICY_SEMVER: &str = "1.0.0";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessResearchEvidenceCommand {
    pub research_run_id: Uuid,
    pub response_id: String,
    pub retrieved_at: DateTime<Utc>,
    pub output: ResearchOutput,
    #[serde(default)]
    pub citations: Vec<WebCitation>,
    #[serde(default)]
    pub sources: Vec<WebSource>,
}

#[derive(Debug, Clone)]
struct SourceReference {
    url: String,
    title: String,
    domain: String,
    independence_key: String,
    tier: String,
    rank: u16,
}

#[derive(Debug, Clone)]
struct PreparedFact {
    fact: ResearchFact,
    normalized_name: String,
    resolution: EntityResolutionRecord,
    time_audit: TimeAuditRecord,
    retrieved_at: DateTime<Utc>,
    sources: Vec<SourceReference>,
    value: Value,
}

#[derive(Debug, Clone)]
struct PersistedFact {
    prepared: PreparedFact,
    evidence_ids: Vec<Uuid>,
    claim_state: EvidenceVerificationState,
}

#[derive(Debug, Clone)]
struct RankedValue {
    key: String,
    value: Value,
    evidence_ids: Vec<Uuid>,
    max_tier_rank: u16,
    independent_domains: usize,
    latest_evidence_at: Option<DateTime<Utc>>,
}

impl ApplicationService {
    pub(super) async fn register_fact_pipeline_artifacts(
        &self,
        store: &football_persistence_postgres::PostgresStore,
    ) -> ApplicationResult<()> {
        store
            .register_source_policy_version(&built_in_source_policy())
            .await?;
        validate_route_registry(&built_in_route_registry())?;
        Ok(())
    }

    pub async fn process_p4_research_evidence(
        &self,
        command: ProcessResearchEvidenceCommand,
    ) -> ApplicationResult<FactPipelineSummary> {
        validate_pipeline_command(&command)?;
        let store = self.active_store().await?;
        let context = store.fact_pipeline_context(command.research_run_id).await?;
        validate_pipeline_context(&command, &context)?;
        let policy = built_in_source_policy().definition;
        let registry = built_in_route_registry();
        let source_index = build_source_index(&command, &policy)?;

        let mut summary = FactPipelineSummary {
            fact_count: u32::try_from(command.output.facts.len()).unwrap_or(u32::MAX),
            missing_field_count: u32::try_from(command.output.missing_fields.len())
                .unwrap_or(u32::MAX),
            ..FactPipelineSummary::default()
        };
        let mut groups: BTreeMap<String, Vec<PreparedFact>> = BTreeMap::new();

        for fact in command.output.facts {
            let route_rule = registry
                .routes
                .iter()
                .find(|route| route.field_key == fact.field_key);
            let normalized_name = normalize_entity_name(&fact.subject.name);
            let compact_name = compact_entity_name(&normalized_name);
            let mut candidates = store
                .find_entity_candidates(
                    &context,
                    &fact.subject.entity_type,
                    &normalized_name,
                    &compact_name,
                    fact.subject.external_id.as_deref(),
                )
                .await?;
            if let Some(side) = route_rule.and_then(|route| route.side.as_deref()) {
                candidates.retain(|candidate| candidate.relation.as_deref() == Some(side));
            }
            let decision =
                if route_rule.is_some_and(|route| route.entity_type != fact.subject.entity_type) {
                    ResolutionDecision {
                        status: EntityResolutionStatus::Unsupported,
                        resolved_entity_id: None,
                        resolved_name: None,
                        strategy: "route_entity_type_mismatch".to_string(),
                        confidence_score: 0,
                        reason: format!(
                            "字段{}要求实体类型{}，联网结果却返回{}",
                            fact.field_key,
                            route_rule
                                .map(|route| route.entity_type.as_str())
                                .unwrap_or("unknown"),
                            fact.subject.entity_type
                        ),
                    }
                } else {
                    decide_entity_resolution(&fact.subject.entity_type, &candidates)
                };
            let resolution_required = route_rule
                .map(|route| route.requires_resolved_entity)
                .unwrap_or(true);
            match decision.status {
                EntityResolutionStatus::Resolved => summary.resolved_entity_count += 1,
                EntityResolutionStatus::Ambiguous if resolution_required => {
                    summary.ambiguous_entity_count += 1;
                }
                EntityResolutionStatus::Unmatched | EntityResolutionStatus::Unsupported
                    if resolution_required =>
                {
                    summary.unmatched_entity_count += 1;
                }
                _ => {}
            }
            let resolution = store
                .append_entity_resolution(&EntityResolutionDraft {
                    research_run_id: context.research_run_id,
                    match_id: context.match_id,
                    trace_id: context.trace_id,
                    fact_key: fact.fact_key.clone(),
                    entity_type: fact.subject.entity_type.clone(),
                    raw_name: fact.subject.name.clone(),
                    normalized_name: normalized_name.clone(),
                    external_id: fact.subject.external_id.clone(),
                    status: decision.status,
                    resolved_entity_id: decision.resolved_entity_id,
                    resolved_name: decision.resolved_name.clone(),
                    strategy: decision.strategy.clone(),
                    confidence_score: decision.confidence_score,
                    candidates,
                    reason: decision.reason.clone(),
                    idempotency_key: format!(
                        "entity:{}:{}",
                        context.research_run_id, fact.fact_key
                    ),
                })
                .await?;

            let (time_status, time_reason) =
                audit_fact_time(&fact, context.data_cutoff_at, command.retrieved_at);
            if !time_status.accepted() {
                summary.time_rejected_count += 1;
            }
            let time_audit = store
                .append_time_audit(&TimeAuditDraft {
                    research_run_id: context.research_run_id,
                    match_id: context.match_id,
                    trace_id: context.trace_id,
                    fact_key: fact.fact_key.clone(),
                    field_key: fact.field_key.clone(),
                    data_cutoff_at: context.data_cutoff_at,
                    published_at: fact.published_at,
                    observed_at: fact.observed_at,
                    effective_at: fact.effective_at,
                    retrieved_at: command.retrieved_at,
                    timezone: fact.timezone.clone(),
                    status: time_status,
                    reason: time_reason,
                    idempotency_key: format!("time:{}:{}", context.research_run_id, fact.fact_key),
                })
                .await?;
            let sources = fact
                .source_urls
                .iter()
                .map(|url| {
                    source_index
                        .get(&normalize_url(url)?)
                        .cloned()
                        .ok_or_else(|| {
                            ApplicationError::Validation(format!(
                                "事实{}的来源没有出现在已验证引用索引中：{}",
                                fact.fact_key, url
                            ))
                        })
                })
                .collect::<ApplicationResult<Vec<_>>>()?;
            let value = serde_json::to_value(&fact.value)?;
            let group_key = format!(
                "{}|{}|{}",
                fact.field_key,
                fact.subject.entity_type,
                resolution
                    .resolved_entity_id
                    .map(|value| value.to_string())
                    .unwrap_or_else(|| normalized_name.clone())
            );
            groups.entry(group_key).or_default().push(PreparedFact {
                fact,
                normalized_name,
                resolution,
                time_audit,
                retrieved_at: command.retrieved_at,
                sources,
                value,
            });
        }

        for prepared_group in groups.into_values() {
            process_fact_group(&store, &context, &registry, prepared_group, &mut summary).await?;
        }

        for missing in command.output.missing_fields {
            process_missing_field(
                &store,
                &context,
                &registry,
                &missing,
                command.retrieved_at,
                &mut summary,
            )
            .await?;
        }
        Ok(summary)
    }
}

async fn process_fact_group(
    store: &football_persistence_postgres::PostgresStore,
    context: &FactPipelineContext,
    registry: &EvidenceRouteRegistry,
    group: Vec<PreparedFact>,
    summary: &mut FactPipelineSummary,
) -> ApplicationResult<()> {
    let field_key = group
        .first()
        .map(|item| item.fact.field_key.clone())
        .ok_or_else(|| ApplicationError::Validation("事实分组不能为空".to_string()))?;
    let accepted_values: BTreeSet<String> = group
        .iter()
        .filter(|item| item.time_audit.status.accepted())
        .map(|item| canonical_json(&item.value))
        .collect::<ApplicationResult<_>>()?;
    let has_conflict = accepted_values.len() > 1
        || group
            .iter()
            .any(|item| item.fact.verification_state == "CONFLICT");
    let independent_by_value = independent_domains_by_value(&group)?;
    let max_rank_by_value = max_rank_by_value(&group)?;

    let mut persisted = Vec::new();
    for prepared in group {
        let value_key = canonical_json(&prepared.value)?;
        let independent_source_count = independent_by_value
            .get(&value_key)
            .copied()
            .unwrap_or_default();
        let max_rank = max_rank_by_value
            .get(&value_key)
            .copied()
            .unwrap_or_default();
        let claim_state =
            determine_claim_state(&prepared, has_conflict, independent_source_count, max_rank);
        let evidence_ids = append_fact_claims(
            store,
            context,
            &prepared,
            claim_state,
            independent_source_count,
        )
        .await?;
        summary.evidence_claim_count = summary
            .evidence_claim_count
            .saturating_add(u32::try_from(evidence_ids.len()).unwrap_or(u32::MAX));
        persisted.push(PersistedFact {
            prepared,
            evidence_ids,
            claim_state,
        });
    }

    let route_rule = registry
        .routes
        .iter()
        .find(|route| route.field_key == field_key)
        .cloned();

    if has_conflict {
        summary.conflict_count += 1;
        process_conflict_group(store, context, route_rule.as_ref(), persisted, summary).await
    } else {
        route_non_conflicting_group(store, context, route_rule.as_ref(), persisted, summary).await
    }
}

async fn append_fact_claims(
    store: &football_persistence_postgres::PostgresStore,
    context: &FactPipelineContext,
    prepared: &PreparedFact,
    state: EvidenceVerificationState,
    independent_source_count: usize,
) -> ApplicationResult<Vec<Uuid>> {
    let sources: Vec<Option<&SourceReference>> = if prepared.sources.is_empty() {
        vec![None]
    } else {
        prepared.sources.iter().map(Some).collect()
    };
    let mut ids = Vec::with_capacity(sources.len());
    for source in sources {
        let source_identity = source
            .map(|value| value.url.as_str())
            .unwrap_or("no-source");
        let idempotency_key = format!(
            "claim:{}:{}",
            context.research_run_id,
            sha256_text(&format!(
                "{}|{}|{}|{}",
                prepared.fact.fact_key,
                source_identity,
                canonical_json(&prepared.value)?,
                state.as_str()
            ))
        );
        let observed_at = prepared
            .fact
            .observed_at
            .or(prepared.fact.published_at)
            .or(prepared.fact.effective_at)
            .unwrap_or(prepared.retrieved_at);
        let record: EvidenceClaimRecord = store
            .append_evidence_claim(&EvidenceClaimDraft {
                match_id: context.match_id,
                entity_type: prepared.fact.subject.entity_type.clone(),
                entity_id: prepared.resolution.resolved_entity_id,
                field_key: prepared.fact.field_key.clone(),
                value: prepared.value.clone(),
                verification_state: state,
                source_tier: source
                    .map(|value| value.tier.clone())
                    .unwrap_or_else(|| "none".to_string()),
                source_document_id: None,
                source_url: source.map(|value| value.url.clone()),
                source_title: source.map(|value| value.title.clone()),
                source_domain: source.map(|value| value.domain.clone()),
                published_at: prepared.fact.published_at,
                observed_at,
                effective_at: prepared.fact.effective_at,
                retrieved_at: prepared.retrieved_at,
                timezone: prepared
                    .fact
                    .timezone
                    .clone()
                    .unwrap_or_else(|| "UNKNOWN".to_string()),
                independent_source_count: u16::try_from(independent_source_count)
                    .unwrap_or(u16::MAX),
                conflict_group_id: None,
                research_run_id: context.research_run_id,
                prompt_version_id: context.prompt_version_id,
                prompt_version: context.prompt_version.clone(),
                schema_version_id: context.schema_version_id,
                schema_version: context.schema_version.clone(),
                idempotency_key,
                metadata: json!({
                    "fact_key": &prepared.fact.fact_key,
                    "raw_verification_state": &prepared.fact.verification_state,
                    "entity_resolution_id": prepared.resolution.id,
                    "entity_resolution_status": prepared.resolution.status.as_str(),
                    "time_audit_id": prepared.time_audit.id,
                    "time_audit_status": prepared.time_audit.status.as_str(),
                    "source_rank": source.map(|value| value.rank),
                    "source_policy_key": SOURCE_POLICY_KEY,
                    "source_policy_version": SOURCE_POLICY_SEMVER,
                    "pipeline_contract": football_domain::P4_FACT_PIPELINE_CONTRACT_VERSION
                }),
            })
            .await?;
        ids.push(record.id);
    }
    Ok(ids)
}

async fn process_conflict_group(
    store: &football_persistence_postgres::PostgresStore,
    context: &FactPipelineContext,
    route_rule: Option<&EvidenceRouteRule>,
    persisted: Vec<PersistedFact>,
    summary: &mut FactPipelineSummary,
) -> ApplicationResult<()> {
    let first = persisted
        .first()
        .ok_or_else(|| ApplicationError::Validation("冲突分组不能为空".to_string()))?;
    let evidence_ids: Vec<Uuid> = persisted
        .iter()
        .filter(|item| item.prepared.time_audit.status.accepted())
        .flat_map(|item| item.evidence_ids.iter().copied())
        .collect();
    if evidence_ids.len() < 2 {
        summary.manual_conflict_count += 1;
        return append_route(
            store,
            context,
            route_rule,
            &persisted,
            EvidenceRouteStatus::BlockedConflict,
            "CONFLICT",
            evidence_ids,
            Value::Null,
            "事实被标记为冲突，但不足两条独立证据形成可评估冲突组",
            summary,
        )
        .await;
    }
    let conflict_key = format!(
        "conflict:{}:{}",
        context.research_run_id,
        sha256_text(&format!(
            "{}|{}|{}",
            first.prepared.fact.field_key,
            first.prepared.fact.subject.entity_type,
            first
                .prepared
                .resolution
                .resolved_entity_id
                .map(|value| value.to_string())
                .unwrap_or_else(|| first.prepared.normalized_name.clone())
        ))
    );
    let conflict = store
        .create_evidence_conflict(&EvidenceConflictDraft {
            match_id: context.match_id,
            entity_type: first.prepared.fact.subject.entity_type.clone(),
            entity_id: first.prepared.resolution.resolved_entity_id,
            field_key: first.prepared.fact.field_key.clone(),
            conflict_key: conflict_key.clone(),
            evidence_ids: evidence_ids.clone(),
            trace_id: context.trace_id,
            metadata: json!({
                "research_run_id": context.research_run_id,
                "pipeline_contract": football_domain::P4_FACT_PIPELINE_CONTRACT_VERSION
            }),
        })
        .await?;
    let ranked = rank_values(&persisted)?;
    let auto_resolved = conflict_winner(&ranked);
    let (evaluation_status, winning, reason) = if let Some(winning) = auto_resolved {
        summary.auto_resolved_conflict_count += 1;
        (
            ConflictEvaluationStatus::AutoResolved,
            Some(winning.clone()),
            "最高来源等级严格高于冲突项，并满足官方来源或独立交叉验证闸门".to_string(),
        )
    } else {
        summary.manual_conflict_count += 1;
        (
            ConflictEvaluationStatus::ManualRequired,
            None,
            "冲突证据在来源等级、独立来源数和时间新鲜度上没有形成安全唯一赢家".to_string(),
        )
    };
    let ranking_payload = serde_json::to_value(
        ranked
            .iter()
            .map(|value| {
                json!({
                    "value": value.value,
                    "max_tier_rank": value.max_tier_rank,
                    "independent_domains": value.independent_domains,
                    "latest_evidence_at": value.latest_evidence_at,
                    "evidence_ids": value.evidence_ids
                })
            })
            .collect::<Vec<_>>(),
    )?;
    store
        .append_conflict_evaluation(&ConflictEvaluationDraft {
            conflict_id: conflict.id,
            research_run_id: context.research_run_id,
            match_id: context.match_id,
            trace_id: context.trace_id,
            source_policy_key: SOURCE_POLICY_KEY.to_string(),
            source_policy_version: SOURCE_POLICY_SEMVER.to_string(),
            status: evaluation_status,
            winning_evidence_ids: winning
                .as_ref()
                .map(|value| value.evidence_ids.clone())
                .unwrap_or_default(),
            winning_value: winning
                .as_ref()
                .map(|value| value.value.clone())
                .unwrap_or(Value::Null),
            ranking: ranking_payload.clone(),
            reason: reason.clone(),
            idempotency_key: format!("evaluation:{}", conflict.id),
        })
        .await?;

    if let Some(winner) = winning {
        store
            .append_conflict_event(
                conflict.id,
                "resolved",
                "deterministic-conflict-resolver",
                &json!({
                    "winning_evidence_ids": winner.evidence_ids.clone(),
                    "winning_value": winner.value.clone(),
                    "ranking": ranking_payload,
                    "reason": reason
                }),
                "deterministic-resolution-v1",
            )
            .await?;
        let route_status = resolved_route_status(route_rule, &persisted, true);
        let route_reason = match route_status {
            EvidenceRouteStatus::Routed => "冲突已由确定性来源等级闸门解决",
            EvidenceRouteStatus::BlockedEntity => {
                "冲突值已解决，但事实主体未通过稳定ID或字段实体类型闸门"
            }
            EvidenceRouteStatus::BlockedUnregisteredField => {
                "冲突值已解决，但字段未登记到唯一模型入口"
            }
            EvidenceRouteStatus::BlockedTime => "冲突值已解决，但没有事实通过赛前时间闸门",
            _ => "冲突值已解决，但事实未进入模型入口",
        };
        append_route(
            store,
            context,
            route_rule,
            &persisted,
            route_status,
            "CONFIRMED",
            winner.evidence_ids,
            winner.value,
            route_reason,
            summary,
        )
        .await
    } else {
        append_route(
            store,
            context,
            route_rule,
            &persisted,
            EvidenceRouteStatus::BlockedConflict,
            "CONFLICT",
            evidence_ids,
            Value::Null,
            "冲突未形成安全唯一赢家，等待后续用户确认",
            summary,
        )
        .await
    }
}

async fn route_non_conflicting_group(
    store: &football_persistence_postgres::PostgresStore,
    context: &FactPipelineContext,
    route_rule: Option<&EvidenceRouteRule>,
    persisted: Vec<PersistedFact>,
    summary: &mut FactPipelineSummary,
) -> ApplicationResult<()> {
    let accepted: Vec<&PersistedFact> = persisted
        .iter()
        .filter(|item| item.prepared.time_audit.status.accepted())
        .collect();
    let all_evidence_ids = accepted
        .iter()
        .flat_map(|item| item.evidence_ids.iter().copied())
        .collect::<Vec<_>>();
    let selected_value = accepted
        .first()
        .map(|item| item.prepared.value.clone())
        .unwrap_or(Value::Null);
    let state = accepted
        .iter()
        .map(|item| item.claim_state)
        .max_by_key(|state| verification_priority(*state))
        .unwrap_or(EvidenceVerificationState::Stale);

    let status = resolved_route_status(route_rule, &persisted, !accepted.is_empty());
    let reason = match status {
        EvidenceRouteStatus::Routed => "事实通过实体、时间、来源和唯一入口验证",
        EvidenceRouteStatus::BlockedTime => "没有通过data_cutoff_at时间闸门的事实",
        EvidenceRouteStatus::BlockedEntity => "事实主体未解析到安全唯一的内部稳定ID",
        EvidenceRouteStatus::BlockedUnregisteredField => "字段未登记到版本化证据路由表",
        _ => "事实未进入模型入口",
    };
    append_route(
        store,
        context,
        route_rule,
        &persisted,
        status,
        state.as_str(),
        all_evidence_ids,
        selected_value,
        reason,
        summary,
    )
    .await
}

fn resolved_route_status(
    route_rule: Option<&EvidenceRouteRule>,
    persisted: &[PersistedFact],
    has_accepted_fact: bool,
) -> EvidenceRouteStatus {
    let Some(rule) = route_rule else {
        return EvidenceRouteStatus::BlockedUnregisteredField;
    };
    if !has_accepted_fact {
        return EvidenceRouteStatus::BlockedTime;
    }
    if persisted
        .iter()
        .filter(|item| item.prepared.time_audit.status.accepted())
        .any(|item| {
            item.prepared.fact.subject.entity_type != rule.entity_type
                || (rule.requires_resolved_entity
                    && item.prepared.resolution.status != EntityResolutionStatus::Resolved)
        })
    {
        EvidenceRouteStatus::BlockedEntity
    } else {
        EvidenceRouteStatus::Routed
    }
}

#[allow(clippy::too_many_arguments)]
async fn append_route(
    store: &football_persistence_postgres::PostgresStore,
    context: &FactPipelineContext,
    route_rule: Option<&EvidenceRouteRule>,
    persisted: &[PersistedFact],
    status: EvidenceRouteStatus,
    verification_state: &str,
    evidence_ids: Vec<Uuid>,
    selected_value: Value,
    reason: &str,
    summary: &mut FactPipelineSummary,
) -> ApplicationResult<()> {
    let first = persisted
        .first()
        .ok_or_else(|| ApplicationError::Validation("证据路由分组不能为空".to_string()))?;
    let entity_id = first.prepared.resolution.resolved_entity_id;
    let entity_component = entity_id
        .map(|value| value.to_string())
        .unwrap_or_else(|| first.prepared.normalized_name.clone());
    let (target_module, target_slot) = route_rule
        .map(|rule| (rule.target_module.clone(), rule.target_slot.clone()))
        .unwrap_or_else(|| {
            (
                "unregistered".to_string(),
                first.prepared.fact.field_key.clone(),
            )
        });
    let route_key = format!("{target_module}:{target_slot}:{entity_component}");
    store
        .append_evidence_route(&EvidenceRouteDraft {
            research_run_id: context.research_run_id,
            match_id: context.match_id,
            trace_id: context.trace_id,
            route_key: route_key.clone(),
            field_key: first.prepared.fact.field_key.clone(),
            target_module,
            target_slot,
            route_registry_version: P4_EVIDENCE_ROUTE_VERSION.to_string(),
            entity_type: Some(first.prepared.fact.subject.entity_type.clone()),
            entity_id,
            status,
            verification_state: verification_state.to_string(),
            selected_evidence_ids: evidence_ids,
            selected_value,
            reason: reason.to_string(),
            idempotency_key: format!(
                "route:{}:{}",
                context.research_run_id,
                sha256_text(&route_key)
            ),
        })
        .await?;
    if status == EvidenceRouteStatus::Routed {
        summary.routed_count += 1;
    } else {
        summary.blocked_count += 1;
    }
    Ok(())
}

async fn process_missing_field(
    store: &football_persistence_postgres::PostgresStore,
    context: &FactPipelineContext,
    registry: &EvidenceRouteRegistry,
    missing: &MissingField,
    retrieved_at: DateTime<Utc>,
    summary: &mut FactPipelineSummary,
) -> ApplicationResult<()> {
    let raw_state = parse_verification_state(&missing.verification_state)?;
    let fact_key = format!("missing.{}", missing.field_key);
    let (time_status, time_reason) = if retrieved_at > context.data_cutoff_at {
        (
            TimeAuditStatus::RejectedRetrievedAfterCutoff,
            "缺失结论在data_cutoff_at之后才取回，已阻止进入对应赛前模型入口".to_string(),
        )
    } else {
        (
            TimeAuditStatus::AcceptedNonFact,
            "字段没有可验证事实，保留明确缺失状态".to_string(),
        )
    };
    if !time_status.accepted() {
        summary.time_rejected_count += 1;
    }
    let claim_state = if time_status.accepted() {
        raw_state
    } else {
        EvidenceVerificationState::Stale
    };
    let time = store
        .append_time_audit(&TimeAuditDraft {
            research_run_id: context.research_run_id,
            match_id: context.match_id,
            trace_id: context.trace_id,
            fact_key: fact_key.clone(),
            field_key: missing.field_key.clone(),
            data_cutoff_at: context.data_cutoff_at,
            published_at: None,
            observed_at: None,
            effective_at: None,
            retrieved_at,
            timezone: None,
            status: time_status,
            reason: time_reason,
            idempotency_key: format!("time:{}:{}", context.research_run_id, fact_key),
        })
        .await?;
    let evidence = store
        .append_evidence_claim(&EvidenceClaimDraft {
            match_id: context.match_id,
            entity_type: "match".to_string(),
            entity_id: Some(context.match_id),
            field_key: missing.field_key.clone(),
            value: Value::Null,
            verification_state: claim_state,
            source_tier: "none".to_string(),
            source_document_id: None,
            source_url: None,
            source_title: None,
            source_domain: None,
            published_at: None,
            observed_at: retrieved_at,
            effective_at: None,
            retrieved_at,
            timezone: "UTC".to_string(),
            independent_source_count: 0,
            conflict_group_id: None,
            research_run_id: context.research_run_id,
            prompt_version_id: context.prompt_version_id,
            prompt_version: context.prompt_version.clone(),
            schema_version_id: context.schema_version_id,
            schema_version: context.schema_version.clone(),
            idempotency_key: format!(
                "claim:{}:{}",
                context.research_run_id,
                sha256_text(&format!(
                    "missing|{}|{}",
                    missing.field_key,
                    claim_state.as_str()
                ))
            ),
            metadata: json!({
                "fact_key": fact_key,
                "time_audit_id": time.id,
                "pipeline_contract": football_domain::P4_FACT_PIPELINE_CONTRACT_VERSION
            }),
        })
        .await?;
    summary.evidence_claim_count += 1;

    let route_rule = registry
        .routes
        .iter()
        .find(|route| route.field_key == missing.field_key);
    let (target_module, target_slot) = route_rule
        .map(|route| (route.target_module.clone(), route.target_slot.clone()))
        .unwrap_or_else(|| ("unregistered".to_string(), missing.field_key.clone()));
    let entity_id = route_rule.and_then(|route| match route.side.as_deref() {
        Some("home") => context.home_team_id,
        Some("away") => context.away_team_id,
        _ if route.entity_type == "competition" => context.competition_id,
        _ => Some(context.match_id),
    });
    let status = if !time_status.accepted() {
        EvidenceRouteStatus::BlockedTime
    } else if route_rule.is_some() {
        EvidenceRouteStatus::Missing
    } else {
        EvidenceRouteStatus::BlockedUnregisteredField
    };
    let route_key = format!("{target_module}:{target_slot}:missing");
    store
        .append_evidence_route(&EvidenceRouteDraft {
            research_run_id: context.research_run_id,
            match_id: context.match_id,
            trace_id: context.trace_id,
            route_key: route_key.clone(),
            field_key: missing.field_key.clone(),
            target_module,
            target_slot,
            route_registry_version: P4_EVIDENCE_ROUTE_VERSION.to_string(),
            entity_type: route_rule.map(|route| route.entity_type.clone()),
            entity_id,
            status,
            verification_state: claim_state.as_str().to_string(),
            selected_evidence_ids: vec![evidence.id],
            selected_value: Value::Null,
            reason: if time_status.accepted() {
                "明确缺失状态进入唯一入口，用于降低数据完整度而不是猜测补齐".to_string()
            } else {
                "缺失结论晚于data_cutoff_at取回，已保留审计但阻止进入赛前模型入口".to_string()
            },
            idempotency_key: format!(
                "route:{}:{}",
                context.research_run_id,
                sha256_text(&route_key)
            ),
        })
        .await?;
    if status == EvidenceRouteStatus::Missing {
        summary.routed_count += 1;
    } else {
        summary.blocked_count += 1;
    }
    Ok(())
}

#[derive(Debug, Clone)]
struct ResolutionDecision {
    status: EntityResolutionStatus,
    resolved_entity_id: Option<Uuid>,
    resolved_name: Option<String>,
    strategy: String,
    confidence_score: u16,
    reason: String,
}

fn decide_entity_resolution(
    entity_type: &str,
    candidates: &[EntityCandidate],
) -> ResolutionDecision {
    if !matches!(entity_type, "match" | "competition" | "team" | "player") {
        return ResolutionDecision {
            status: EntityResolutionStatus::Unsupported,
            resolved_entity_id: None,
            resolved_name: None,
            strategy: "no_stable_catalog".to_string(),
            confidence_score: 0,
            reason: "当前实体类型没有稳定目录；保留原始名称，不伪造内部ID".to_string(),
        };
    }
    let Some(top) = candidates.first() else {
        return ResolutionDecision {
            status: EntityResolutionStatus::Unmatched,
            resolved_entity_id: None,
            resolved_name: None,
            strategy: "no_exact_candidate".to_string(),
            confidence_score: 0,
            reason: "未找到符合比赛时点与别名有效期的精确候选".to_string(),
        };
    };
    let tied = candidates
        .iter()
        .skip(1)
        .any(|candidate| candidate.score == top.score && candidate.entity_id != top.entity_id);
    if tied {
        return ResolutionDecision {
            status: EntityResolutionStatus::Ambiguous,
            resolved_entity_id: None,
            resolved_name: None,
            strategy: "equal_score_candidates".to_string(),
            confidence_score: top.score.min(100),
            reason: "多个内部实体获得相同最高匹配分，不能静默选择".to_string(),
        };
    }
    if top.score < 90 {
        return ResolutionDecision {
            status: EntityResolutionStatus::Unmatched,
            resolved_entity_id: None,
            resolved_name: None,
            strategy: top.strategy.clone(),
            confidence_score: top.score.min(100),
            reason: "仅找到比赛范围外候选，未达到自动解析安全阈值".to_string(),
        };
    }
    ResolutionDecision {
        status: EntityResolutionStatus::Resolved,
        resolved_entity_id: Some(top.entity_id),
        resolved_name: Some(top.canonical_name.clone()),
        strategy: top.strategy.clone(),
        confidence_score: top.score.min(100),
        reason: "候选在比赛范围、别名有效期或外部ID上形成安全唯一匹配".to_string(),
    }
}

fn audit_fact_time(
    fact: &ResearchFact,
    cutoff: DateTime<Utc>,
    retrieved_at: DateTime<Utc>,
) -> (TimeAuditStatus, String) {
    if retrieved_at > cutoff {
        return (
            TimeAuditStatus::RejectedRetrievedAfterCutoff,
            "API结果在data_cutoff_at之后才取回，已阻止进入对应赛前模型入口".to_string(),
        );
    }
    if matches!(
        fact.verification_state.as_str(),
        "NOT_FOUND" | "NOT_APPLICABLE"
    ) {
        return (
            TimeAuditStatus::AcceptedNonFact,
            "非事实结论不要求来源时间".to_string(),
        );
    }
    if fact
        .timezone
        .as_deref()
        .is_none_or(|value| value.trim().is_empty())
    {
        return (
            TimeAuditStatus::RejectedMissingTimezone,
            "有事实结论但缺少来源时区".to_string(),
        );
    }
    let timestamps = [fact.published_at, fact.observed_at, fact.effective_at];
    if timestamps.iter().all(Option::is_none) {
        return (
            TimeAuditStatus::RejectedMissingEvidenceTime,
            "有事实结论但published_at、observed_at和effective_at均为空".to_string(),
        );
    }
    if timestamps.iter().flatten().any(|value| *value > cutoff) {
        return (
            TimeAuditStatus::RejectedFuture,
            "事实时间晚于data_cutoff_at，已阻止进入赛前模型入口".to_string(),
        );
    }
    if fact
        .published_at
        .into_iter()
        .chain(fact.observed_at)
        .any(|value| value > retrieved_at)
    {
        return (
            TimeAuditStatus::RejectedInvalidOrder,
            "发布时间或观察时间晚于实际抓取时间".to_string(),
        );
    }
    (
        TimeAuditStatus::Accepted,
        "事实时间不晚于data_cutoff_at且具备明确时区".to_string(),
    )
}

fn build_source_index(
    command: &ProcessResearchEvidenceCommand,
    policy: &SourcePolicyDefinition,
) -> ApplicationResult<BTreeMap<String, SourceReference>> {
    let mut index = BTreeMap::new();
    for source in &command.sources {
        let (normalized, domain) = normalize_source_url(&source.url)?;
        let (tier, rank, independence_key) = classify_source(&domain, policy)?;
        index.insert(
            normalized,
            SourceReference {
                url: source.url.clone(),
                title: source.title.clone().unwrap_or_else(|| domain.clone()),
                domain,
                independence_key,
                tier,
                rank,
            },
        );
    }
    for citation in &command.citations {
        let (normalized, domain) = normalize_source_url(&citation.url)?;
        let (tier, rank, independence_key) = classify_source(&domain, policy)?;
        index.insert(
            normalized,
            SourceReference {
                url: citation.url.clone(),
                title: citation.title.clone(),
                domain,
                independence_key,
                tier,
                rank,
            },
        );
    }
    Ok(index)
}

fn classify_source(
    domain: &str,
    policy: &SourcePolicyDefinition,
) -> ApplicationResult<(String, u16, String)> {
    let host = normalize_domain(domain);
    let matched_rule = policy
        .domain_rules
        .iter()
        .filter(|rule| domain_matches(&host, &rule.domain))
        .max_by_key(|rule| normalize_domain(&rule.domain).len());
    let tier = matched_rule
        .map(|rule| rule.tier.as_str())
        .unwrap_or(policy.default_tier.as_str());
    let rank = policy
        .tiers
        .iter()
        .find(|definition| definition.key == tier)
        .map(|definition| definition.rank)
        .ok_or_else(|| ApplicationError::Validation(format!("来源策略引用了未定义等级：{tier}")))?;
    let independence_key = matched_rule
        .map(|rule| normalize_domain(&rule.domain))
        .unwrap_or_else(|| host.clone());
    Ok((tier.to_string(), rank, independence_key))
}

fn domain_matches(host: &str, configured: &str) -> bool {
    let configured = normalize_domain(configured);
    host == configured || host.ends_with(&format!(".{configured}"))
}

fn normalize_domain(value: &str) -> String {
    value
        .trim()
        .trim_start_matches("*.")
        .trim_start_matches("www.")
        .to_lowercase()
}

fn normalize_source_url(value: &str) -> ApplicationResult<(String, String)> {
    let mut url = Url::parse(value)
        .map_err(|_| ApplicationError::Validation(format!("无法规范化来源URL：{value}")))?;
    if url.scheme() != "https" || !url.username().is_empty() || url.password().is_some() {
        return Err(ApplicationError::Validation(format!(
            "来源URL必须使用HTTPS且不能包含用户名或密码：{value}"
        )));
    }
    let domain = url
        .host_str()
        .map(normalize_domain)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| ApplicationError::Validation(format!("来源URL缺少有效域名：{value}")))?;
    url.set_fragment(None);
    Ok((url.to_string(), domain))
}

fn normalize_url(value: &str) -> ApplicationResult<String> {
    normalize_source_url(value).map(|(url, _)| url)
}

fn normalize_entity_name(value: &str) -> String {
    value
        .trim()
        .to_lowercase()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn compact_entity_name(value: &str) -> String {
    value
        .chars()
        .filter(|character| character.is_alphanumeric())
        .collect()
}

fn independent_domains_by_value(
    group: &[PreparedFact],
) -> ApplicationResult<BTreeMap<String, usize>> {
    let mut values: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for item in group
        .iter()
        .filter(|item| item.time_audit.status.accepted())
    {
        let key = canonical_json(&item.value)?;
        let domains = values.entry(key).or_default();
        for source in &item.sources {
            domains.insert(source.independence_key.clone());
        }
    }
    Ok(values
        .into_iter()
        .map(|(key, domains)| (key, domains.len()))
        .collect())
}

fn max_rank_by_value(group: &[PreparedFact]) -> ApplicationResult<BTreeMap<String, u16>> {
    let mut values = BTreeMap::new();
    for item in group
        .iter()
        .filter(|item| item.time_audit.status.accepted())
    {
        let key = canonical_json(&item.value)?;
        let rank = item
            .sources
            .iter()
            .map(|source| source.rank)
            .max()
            .unwrap_or(0);
        values
            .entry(key)
            .and_modify(|current| *current = std::cmp::max(*current, rank))
            .or_insert(rank);
    }
    Ok(values)
}

fn determine_claim_state(
    prepared: &PreparedFact,
    has_conflict: bool,
    independent_source_count: usize,
    max_rank: u16,
) -> EvidenceVerificationState {
    if !prepared.time_audit.status.accepted() {
        return EvidenceVerificationState::Stale;
    }
    if has_conflict || prepared.fact.verification_state == "CONFLICT" {
        return EvidenceVerificationState::Conflict;
    }
    if prepared.fact.verification_state == "STALE" {
        return EvidenceVerificationState::Stale;
    }
    if prepared.fact.verification_state == "NOT_APPLICABLE" {
        return EvidenceVerificationState::NotApplicable;
    }
    if prepared.fact.verification_state == "NOT_FOUND" {
        return EvidenceVerificationState::NotFound;
    }
    if max_rank >= 450 || independent_source_count >= 2 {
        EvidenceVerificationState::Confirmed
    } else {
        EvidenceVerificationState::Probable
    }
}

fn rank_values(persisted: &[PersistedFact]) -> ApplicationResult<Vec<RankedValue>> {
    let mut values: BTreeMap<String, RankedValue> = BTreeMap::new();
    for item in persisted
        .iter()
        .filter(|item| item.prepared.time_audit.status.accepted())
    {
        let key = canonical_json(&item.prepared.value)?;
        let entry = values.entry(key.clone()).or_insert_with(|| RankedValue {
            key,
            value: item.prepared.value.clone(),
            evidence_ids: Vec::new(),
            max_tier_rank: 0,
            independent_domains: 0,
            latest_evidence_at: None,
        });
        entry.evidence_ids.extend(item.evidence_ids.iter().copied());
        entry.max_tier_rank = entry.max_tier_rank.max(
            item.prepared
                .sources
                .iter()
                .map(|source| source.rank)
                .max()
                .unwrap_or(0),
        );
        let domains: BTreeSet<_> = persisted
            .iter()
            .filter(|candidate| candidate.prepared.time_audit.status.accepted())
            .filter(|candidate| {
                canonical_json(&candidate.prepared.value).ok().as_deref()
                    == Some(entry.key.as_str())
            })
            .flat_map(|candidate| {
                candidate
                    .prepared
                    .sources
                    .iter()
                    .map(|source| source.independence_key.clone())
            })
            .collect();
        entry.independent_domains = domains.len();
        let latest = item
            .prepared
            .fact
            .effective_at
            .or(item.prepared.fact.observed_at)
            .or(item.prepared.fact.published_at);
        if latest > entry.latest_evidence_at {
            entry.latest_evidence_at = latest;
        }
    }
    let mut ranked: Vec<_> = values.into_values().collect();
    ranked.sort_by(compare_ranked_values);
    Ok(ranked)
}

fn compare_ranked_values(left: &RankedValue, right: &RankedValue) -> Ordering {
    right
        .max_tier_rank
        .cmp(&left.max_tier_rank)
        .then_with(|| right.independent_domains.cmp(&left.independent_domains))
        .then_with(|| right.latest_evidence_at.cmp(&left.latest_evidence_at))
        .then_with(|| left.key.cmp(&right.key))
}

fn conflict_winner(ranked: &[RankedValue]) -> Option<&RankedValue> {
    let top = ranked.first()?;
    let second = ranked.get(1)?;
    let official_unique = top.max_tier_rank >= 450 && top.max_tier_rank > second.max_tier_rank;
    let independently_confirmed = top.max_tier_rank >= 250
        && top.independent_domains >= 2
        && top.max_tier_rank > second.max_tier_rank
        && top.independent_domains > second.independent_domains;
    (official_unique || independently_confirmed).then_some(top)
}

fn verification_priority(state: EvidenceVerificationState) -> u8 {
    match state {
        EvidenceVerificationState::Confirmed => 6,
        EvidenceVerificationState::Probable => 5,
        EvidenceVerificationState::Conflict => 4,
        EvidenceVerificationState::Stale => 3,
        EvidenceVerificationState::NotFound => 2,
        EvidenceVerificationState::NotApplicable => 1,
    }
}

fn parse_verification_state(value: &str) -> ApplicationResult<EvidenceVerificationState> {
    match value {
        "CONFIRMED" => Ok(EvidenceVerificationState::Confirmed),
        "PROBABLE" => Ok(EvidenceVerificationState::Probable),
        "CONFLICT" => Ok(EvidenceVerificationState::Conflict),
        "NOT_FOUND" => Ok(EvidenceVerificationState::NotFound),
        "STALE" => Ok(EvidenceVerificationState::Stale),
        "NOT_APPLICABLE" => Ok(EvidenceVerificationState::NotApplicable),
        other => Err(ApplicationError::Validation(format!(
            "未知证据状态：{other}"
        ))),
    }
}

fn canonical_json(value: &Value) -> ApplicationResult<String> {
    Ok(serde_json::to_string(value)?)
}

fn sha256_text(value: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(value.as_bytes());
    hex::encode(hasher.finalize())
}

fn built_in_source_policy() -> SourcePolicyVersionDraft {
    SourcePolicyVersionDraft {
        policy_key: SOURCE_POLICY_KEY.to_string(),
        version: SOURCE_POLICY_SEMVER.to_string(),
        competition_profile_id: None,
        definition: serde_json::from_str(include_str!(
            "../../../src-tauri/resources/research/public_source_policy.json"
        ))
        .expect("内置P4来源策略必须有效"),
        metadata: json!({
            "stage": "E",
            "schema_version": P4_SOURCE_POLICY_VERSION,
            "competition_override_supported": true
        }),
    }
}

fn built_in_route_registry() -> EvidenceRouteRegistry {
    serde_json::from_str(include_str!(
        "../../../src-tauri/resources/research/public_evidence_routes.json"
    ))
    .expect("内置P4证据路由注册表必须有效")
}

fn validate_route_registry(registry: &EvidenceRouteRegistry) -> ApplicationResult<()> {
    if registry.schema_version != P4_EVIDENCE_ROUTE_VERSION {
        return Err(ApplicationError::Validation(
            "证据路由注册表Schema版本不匹配".to_string(),
        ));
    }
    let mut fields = BTreeSet::new();
    let mut slots = BTreeSet::new();
    for route in &registry.routes {
        if route.field_key.trim().is_empty()
            || route.target_module.trim().is_empty()
            || route.target_slot.trim().is_empty()
            || !fields.insert(route.field_key.as_str())
            || !slots.insert((route.target_module.as_str(), route.target_slot.as_str()))
        {
            return Err(ApplicationError::Validation(
                "证据路由必须具有唯一字段和唯一模型入口".to_string(),
            ));
        }
        if route
            .side
            .as_deref()
            .is_some_and(|side| !matches!(side, "home" | "away"))
        {
            return Err(ApplicationError::Validation(
                "证据路由side只能是home或away".to_string(),
            ));
        }
    }
    Ok(())
}

fn validate_pipeline_command(command: &ProcessResearchEvidenceCommand) -> ApplicationResult<()> {
    if command.response_id.trim().is_empty() || command.response_id.chars().count() > 200 {
        return Err(ApplicationError::Validation(
            "事实流水线必须关联有效response_id".to_string(),
        ));
    }
    Ok(())
}

fn validate_pipeline_context(
    command: &ProcessResearchEvidenceCommand,
    context: &FactPipelineContext,
) -> ApplicationResult<()> {
    if command.output.match_key != context.match_key
        || command.output.data_cutoff_at != context.data_cutoff_at
        || command.output.schema_version != context.schema_version
    {
        return Err(ApplicationError::Validation(
            "联网结果与研究任务的比赛键、截止时间或Schema版本不一致".to_string(),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
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
        let index = build_source_index(&command, &built_in_source_policy().definition)
            .expect("source index");
        let source = index.values().next().expect("source");
        assert_eq!(source.domain, "news.example");
        assert_eq!(source.tier, "unclassified");
    }
}
