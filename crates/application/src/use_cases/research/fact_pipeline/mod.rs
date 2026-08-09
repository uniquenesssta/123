use crate::ports::research::{
    FactPipelinePort, ResearchArtifactPort, ResearchEvidenceLedgerPort,
    SerializedConflictEventPayload,
};
use crate::{ApplicationError, ApplicationResult};
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

mod conflict;
mod entity_resolution;
mod evidence;
mod routing;
mod source_policy;
mod time_audit;
mod types;
mod validation;

use conflict::*;
use entity_resolution::*;
use evidence::*;
use routing::*;
use source_policy::*;
use time_audit::*;
use types::*;
use validation::*;

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

pub(crate) trait FactPipelineAccess: FactPipelinePort + ResearchEvidenceLedgerPort {}
impl<T> FactPipelineAccess for T where T: FactPipelinePort + ResearchEvidenceLedgerPort + ?Sized {}

pub(crate) async fn register_fact_pipeline_artifacts(
    port: &dyn ResearchArtifactPort,
) -> ApplicationResult<()> {
    port.register_source_policy(&built_in_source_policy())
        .await?;
    validate_route_registry(&built_in_route_registry())?;
    Ok(())
}

pub(crate) async fn process_p4_research_evidence(
    port: &dyn FactPipelineAccess,
    command: ProcessResearchEvidenceCommand,
) -> ApplicationResult<FactPipelineSummary> {
    validate_pipeline_command(&command)?;
    let context = port.context(command.research_run_id).await?;
    validate_pipeline_context(&command, &context)?;
    let policy = built_in_source_policy().definition;
    let registry = built_in_route_registry();
    let source_index = build_source_index(&command, &policy)?;

    let mut summary = FactPipelineSummary {
        fact_count: u32::try_from(command.output.facts.len()).unwrap_or(u32::MAX),
        missing_field_count: u32::try_from(command.output.missing_fields.len()).unwrap_or(u32::MAX),
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
        let mut candidates = port
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

        let (time_status, time_reason) =
            audit_fact_time(&fact, context.data_cutoff_at, command.retrieved_at);
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
        process_fact_group(port, &context, &registry, prepared_group, &mut summary).await?;
    }

    for missing in command.output.missing_fields {
        process_missing_field(
            port,
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

#[cfg(test)]
mod tests;
