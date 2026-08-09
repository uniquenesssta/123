use super::*;

pub(super) async fn process_conflict_group(
    port: &dyn FactPipelineAccess,
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
            port,
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
    let conflict = port
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
    port.append_conflict_evaluation(&ConflictEvaluationDraft {
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
        let conflict_event_payload =
            SerializedConflictEventPayload(serde_json::to_string(&json!({
                "winning_evidence_ids": winner.evidence_ids.clone(),
                "winning_value": winner.value.clone(),
                "ranking": ranking_payload,
                "reason": reason
            }))?);
        port.append_conflict_event(
            conflict.id,
            "resolved",
            "deterministic-conflict-resolver",
            &conflict_event_payload,
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
            port,
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
            port,
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

pub(super) fn rank_values(persisted: &[PersistedFact]) -> ApplicationResult<Vec<RankedValue>> {
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

pub(super) fn compare_ranked_values(left: &RankedValue, right: &RankedValue) -> Ordering {
    right
        .max_tier_rank
        .cmp(&left.max_tier_rank)
        .then_with(|| right.independent_domains.cmp(&left.independent_domains))
        .then_with(|| right.latest_evidence_at.cmp(&left.latest_evidence_at))
        .then_with(|| left.key.cmp(&right.key))
}

pub(super) fn conflict_winner(ranked: &[RankedValue]) -> Option<&RankedValue> {
    let top = ranked.first()?;
    let second = ranked.get(1)?;
    let official_unique = top.max_tier_rank >= 450 && top.max_tier_rank > second.max_tier_rank;
    let independently_confirmed = top.max_tier_rank >= 250
        && top.independent_domains >= 2
        && top.max_tier_rank > second.max_tier_rank
        && top.independent_domains > second.independent_domains;
    (official_unique || independently_confirmed).then_some(top)
}
