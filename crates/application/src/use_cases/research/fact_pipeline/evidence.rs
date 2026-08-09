use super::*;

pub(super) async fn process_fact_group(
    port: &dyn FactPipelineAccess,
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
            port,
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
        process_conflict_group(port, context, route_rule.as_ref(), persisted, summary).await
    } else {
        route_non_conflicting_group(port, context, route_rule.as_ref(), persisted, summary).await
    }
}

pub(super) async fn append_fact_claims(
    port: &dyn FactPipelineAccess,
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
        let record: EvidenceClaimRecord = port
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

pub(super) fn independent_domains_by_value(
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

pub(super) fn max_rank_by_value(
    group: &[PreparedFact],
) -> ApplicationResult<BTreeMap<String, u16>> {
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

pub(super) fn determine_claim_state(
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
