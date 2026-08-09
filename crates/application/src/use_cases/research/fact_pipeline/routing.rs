use super::*;

pub(super) async fn route_non_conflicting_group(
    port: &dyn FactPipelineAccess,
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
        port,
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

pub(super) fn resolved_route_status(
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
pub(super) async fn append_route(
    port: &dyn FactPipelineAccess,
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
    port.append_evidence_route(&EvidenceRouteDraft {
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

pub(super) async fn process_missing_field(
    port: &dyn FactPipelineAccess,
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
    let time = port
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
    let evidence = port
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
    port.append_evidence_route(&EvidenceRouteDraft {
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

pub(super) fn built_in_route_registry() -> EvidenceRouteRegistry {
    serde_json::from_str(include_str!(
        "../../../../../../src-tauri/resources/research/public_evidence_routes.json"
    ))
    .expect("内置P4证据路由注册表必须有效")
}

pub(super) fn validate_route_registry(registry: &EvidenceRouteRegistry) -> ApplicationResult<()> {
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
