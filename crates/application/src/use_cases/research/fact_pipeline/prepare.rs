use super::entity_resolution::{
    compact_entity_name, decide_entity_resolution, normalize_entity_name,
};
use super::process::FactPipelineAccess;
use super::source_policy::normalize_url;
use super::time_audit::audit_fact_time;
use super::types::{PreparedFact, ResolutionDecision, SourceReference};
use crate::{ApplicationError, ApplicationResult};
use chrono::{DateTime, Utc};
use football_domain::{
    EntityResolutionDraft, EntityResolutionStatus, EvidenceRouteRegistry, FactPipelineContext,
    FactPipelineSummary, TimeAuditDraft,
};
use football_research_gateway::ResearchFact;
use std::collections::BTreeMap;

pub(super) async fn prepare_fact(
    port: &dyn FactPipelineAccess,
    context: &FactPipelineContext,
    registry: &EvidenceRouteRegistry,
    source_index: &BTreeMap<String, SourceReference>,
    fact: ResearchFact,
    retrieved_at: DateTime<Utc>,
    summary: &mut FactPipelineSummary,
) -> ApplicationResult<(String, PreparedFact)> {
    let route_rule = registry
        .routes
        .iter()
        .find(|route| route.field_key == fact.field_key);
    let normalized_name = normalize_entity_name(&fact.subject.name);
    let compact_name = compact_entity_name(&normalized_name);
    let mut candidates = port
        .find_entity_candidates(
            context,
            &fact.subject.entity_type,
            &normalized_name,
            &compact_name,
            fact.subject.external_id.as_deref(),
        )
        .await?;
    if let Some(side) = route_rule.and_then(|route| route.side.as_deref()) {
        candidates.retain(|candidate| candidate.relation.as_deref() == Some(side));
    }
    let decision = if route_rule.is_some_and(|route| route.entity_type != fact.subject.entity_type)
    {
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
    let resolution = port
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
            idempotency_key: format!("entity:{}:{}", context.research_run_id, fact.fact_key),
        })
        .await?;

    let (time_status, time_reason) = audit_fact_time(&fact, context.data_cutoff_at, retrieved_at);
    if !time_status.accepted() {
        summary.time_rejected_count += 1;
    }
    let time_audit = port
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
            retrieved_at,
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
    Ok((
        group_key,
        PreparedFact {
            fact,
            normalized_name,
            resolution,
            time_audit,
            retrieved_at,
            sources,
            value,
        },
    ))
}
