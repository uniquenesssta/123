use super::*;

pub(super) fn decide_entity_resolution(
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

pub(super) fn normalize_entity_name(value: &str) -> String {
    value
        .trim()
        .to_lowercase()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

pub(super) fn compact_entity_name(value: &str) -> String {
    value
        .chars()
        .filter(|character| character.is_alphanumeric())
        .collect()
}
