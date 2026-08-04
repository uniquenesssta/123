use crate::{
    GatewayError, GatewayErrorCategory, ResearchOutput, ResearchValue, ResearchValueKind,
    SourcePolicy, WebCitation, WebSource,
};
use chrono::{DateTime, Utc};
use std::collections::{BTreeMap, BTreeSet};
use url::Url;

pub struct ValidationContext<'a> {
    pub match_key: &'a str,
    pub schema_version: &'a str,
    pub data_cutoff_at: DateTime<Utc>,
    pub requested_fact_keys: &'a [String],
    pub source_policy: &'a SourcePolicy,
    pub citations: &'a [WebCitation],
    pub sources: &'a [WebSource],
}

pub fn validate_research_output(
    output: &ResearchOutput,
    context: &ValidationContext<'_>,
) -> Result<(), GatewayError> {
    if output.match_key != context.match_key {
        return Err(schema_error("联网输出的比赛键与研究任务不一致"));
    }
    if output.schema_version != context.schema_version {
        return Err(schema_error("联网输出的Schema版本与研究任务不一致"));
    }
    if output.data_cutoff_at != context.data_cutoff_at {
        return Err(schema_error("联网输出的数据截止时间与研究任务不一致"));
    }

    let requested: BTreeSet<&str> = context
        .requested_fact_keys
        .iter()
        .map(String::as_str)
        .collect();
    let prohibited_keys: Vec<String> = context
        .source_policy
        .prohibited_fact_keys
        .iter()
        .map(|value| value.to_lowercase())
        .collect();
    let prohibited_terms: Vec<String> = context
        .source_policy
        .prohibited_content_terms
        .iter()
        .map(|value| value.to_lowercase())
        .collect();
    let source_index = build_source_index(context)?;
    let mut seen_fact_keys = BTreeSet::new();
    let mut seen_fields = BTreeSet::new();

    if output.facts.len() > 256 {
        return Err(schema_error("联网输出的原子事实数量超过256条上限"));
    }

    for fact in &output.facts {
        if !requested.contains(fact.field_key.as_str()) {
            return Err(schema_error(format!(
                "联网输出包含未请求字段：{}",
                fact.field_key
            )));
        }
        if fact.fact_key.trim().is_empty()
            || fact.fact_key.chars().count() > 120
            || !fact.fact_key.bytes().all(|byte| {
                byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'.' | b':' | b'-')
            })
        {
            return Err(schema_error(format!(
                "事实{}的fact_key格式无效",
                fact.field_key
            )));
        }
        if !seen_fact_keys.insert(fact.fact_key.as_str()) {
            return Err(schema_error(format!(
                "联网输出包含重复fact_key：{}",
                fact.fact_key
            )));
        }
        seen_fields.insert(fact.field_key.as_str());
        let normalized_key = fact.field_key.to_lowercase();
        if prohibited_keys
            .iter()
            .any(|term| normalized_key.contains(term))
        {
            return Err(source_error(format!(
                "字段{}属于禁止进入模型的预测、盘口或推荐内容",
                fact.field_key
            )));
        }
        validate_subject(&fact.subject.entity_type, &fact.subject.name)?;
        validate_verification_state(&fact.verification_state)?;
        validate_value(&fact.value, &prohibited_terms)?;
        validate_time("published_at", fact.published_at, context.data_cutoff_at)?;
        validate_time("observed_at", fact.observed_at, context.data_cutoff_at)?;
        validate_time("effective_at", fact.effective_at, context.data_cutoff_at)?;
        validate_fact_sources(
            &fact.verification_state,
            &fact.source_urls,
            &source_index,
            context.source_policy,
        )?;
    }

    let mut missing_seen = BTreeSet::new();
    for missing in &output.missing_fields {
        if !requested.contains(missing.field_key.as_str()) {
            return Err(schema_error(format!(
                "缺失字段列表包含未请求字段：{}",
                missing.field_key
            )));
        }
        if seen_fields.contains(missing.field_key.as_str())
            || !missing_seen.insert(missing.field_key.as_str())
        {
            return Err(schema_error(format!(
                "字段{}同时出现在事实或重复缺失列表中",
                missing.field_key
            )));
        }
        if !matches!(
            missing.verification_state.as_str(),
            "NOT_FOUND" | "STALE" | "NOT_APPLICABLE"
        ) {
            return Err(schema_error(format!(
                "缺失字段{}使用了无效状态{}",
                missing.field_key, missing.verification_state
            )));
        }
    }

    for key in requested {
        if !seen_fields.contains(key) && !missing_seen.contains(key) {
            return Err(schema_error(format!(
                "请求字段{key}既没有事实结果，也没有明确缺失状态"
            )));
        }
    }
    Ok(())
}

fn build_source_index(
    context: &ValidationContext<'_>,
) -> Result<BTreeMap<String, String>, GatewayError> {
    let mut index = BTreeMap::new();
    for source in context.sources {
        let normalized = validate_url(&source.url, context.source_policy)?;
        index.insert(normalized, source.domain.to_lowercase());
    }
    for citation in context.citations {
        let normalized = validate_url(&citation.url, context.source_policy)?;
        index.insert(normalized, citation.domain.to_lowercase());
    }
    Ok(index)
}

fn validate_fact_sources(
    state: &str,
    urls: &[String],
    source_index: &BTreeMap<String, String>,
    policy: &SourcePolicy,
) -> Result<(), GatewayError> {
    let requires_source = !matches!(state, "NOT_FOUND" | "NOT_APPLICABLE");
    if requires_source && urls.is_empty() {
        return Err(source_error("有事实结论的字段缺少Web Search来源"));
    }
    let mut seen = BTreeSet::new();
    for source_url in urls {
        let normalized = validate_url(source_url, policy)?;
        if !seen.insert(normalized.clone()) {
            return Err(source_error("同一事实包含重复来源URL"));
        }
        if !source_index.contains_key(&normalized) {
            return Err(source_error(format!(
                "事实来源未出现在Responses API的引用或完整搜索来源清单中：{source_url}"
            )));
        }
    }
    Ok(())
}

fn validate_url(value: &str, policy: &SourcePolicy) -> Result<String, GatewayError> {
    let parsed = Url::parse(value).map_err(|_| source_error(format!("无效来源URL：{value}")))?;
    if policy.https_only && parsed.scheme() != "https" {
        return Err(source_error(format!("来源URL必须使用HTTPS：{value}")));
    }
    let host = parsed
        .host_str()
        .ok_or_else(|| source_error(format!("来源URL缺少域名：{value}")))?
        .trim_start_matches("www.")
        .to_lowercase();
    if policy
        .blocked_domains
        .iter()
        .any(|domain| domain_matches(&host, domain))
    {
        return Err(source_error(format!("来源域名已被禁止：{host}")));
    }
    if !policy.allowed_domains.is_empty()
        && !policy
            .allowed_domains
            .iter()
            .any(|domain| domain_matches(&host, domain))
    {
        return Err(source_error(format!(
            "来源域名不在当前赛事允许列表：{host}"
        )));
    }
    let mut normalized = parsed;
    normalized.set_fragment(None);
    Ok(normalized.to_string())
}

fn domain_matches(host: &str, configured: &str) -> bool {
    let configured = configured
        .trim()
        .trim_start_matches("*.")
        .trim_start_matches("www.")
        .to_lowercase();
    host == configured || host.ends_with(&format!(".{configured}"))
}

fn validate_subject(entity_type: &str, name: &str) -> Result<(), GatewayError> {
    if !matches!(
        entity_type,
        "match" | "competition" | "venue" | "team" | "player" | "coach" | "official"
    ) {
        return Err(schema_error(format!("未知事实实体类型：{entity_type}")));
    }
    if name.trim().is_empty() || name.chars().count() > 200 {
        return Err(schema_error("事实主体名称不能为空且不能超过200个字符"));
    }
    Ok(())
}

fn validate_verification_state(state: &str) -> Result<(), GatewayError> {
    if matches!(
        state,
        "CONFIRMED" | "PROBABLE" | "CONFLICT" | "NOT_FOUND" | "STALE" | "NOT_APPLICABLE"
    ) {
        Ok(())
    } else {
        Err(schema_error(format!("未知事实验证状态：{state}")))
    }
}

fn validate_value(value: &ResearchValue, prohibited_terms: &[String]) -> Result<(), GatewayError> {
    let valid = match value.kind {
        ResearchValueKind::String => {
            value
                .text
                .as_deref()
                .is_some_and(|text| !text.trim().is_empty())
                && value.number.is_none()
                && value.integer.is_none()
                && value.boolean.is_none()
                && value.strings.is_empty()
        }
        ResearchValueKind::Number => {
            value.number.is_some_and(f64::is_finite)
                && value.text.is_none()
                && value.integer.is_none()
                && value.boolean.is_none()
                && value.strings.is_empty()
        }
        ResearchValueKind::Integer => {
            value.integer.is_some()
                && value.text.is_none()
                && value.number.is_none()
                && value.boolean.is_none()
                && value.strings.is_empty()
        }
        ResearchValueKind::Boolean => {
            value.boolean.is_some()
                && value.text.is_none()
                && value.number.is_none()
                && value.integer.is_none()
                && value.strings.is_empty()
        }
        ResearchValueKind::StringList => {
            !value.strings.is_empty()
                && value.strings.iter().all(|item| !item.trim().is_empty())
                && value.text.is_none()
                && value.number.is_none()
                && value.integer.is_none()
                && value.boolean.is_none()
        }
        ResearchValueKind::Null => {
            value.text.is_none()
                && value.number.is_none()
                && value.integer.is_none()
                && value.boolean.is_none()
                && value.strings.is_empty()
        }
    };
    if !valid {
        return Err(schema_error("事实值kind与实际载荷不一致"));
    }
    let text = match value.kind {
        ResearchValueKind::String => value.text.as_deref().unwrap_or_default().to_lowercase(),
        ResearchValueKind::StringList => value.strings.join(" ").to_lowercase(),
        _ => String::new(),
    };
    if prohibited_terms.iter().any(|term| text.contains(term)) {
        return Err(source_error(
            "联网结果包含预测、推荐、盘口或博彩分析内容，已阻止进入事实库",
        ));
    }
    Ok(())
}

fn validate_time(
    field: &str,
    value: Option<DateTime<Utc>>,
    cutoff: DateTime<Utc>,
) -> Result<(), GatewayError> {
    if value.is_some_and(|timestamp| timestamp > cutoff) {
        return Err(schema_error(format!(
            "{field}晚于赛前数据截止时间，不能进入当前研究结果"
        )));
    }
    Ok(())
}

fn schema_error(message: impl Into<String>) -> GatewayError {
    GatewayError::new(
        GatewayErrorCategory::SchemaValidation,
        message,
        false,
        "保留原始响应并将任务标记为结构校验失败；修正Prompt或Schema版本后重试",
    )
}

fn source_error(message: impl Into<String>) -> GatewayError {
    GatewayError::new(
        GatewayErrorCategory::SourcePolicy,
        message,
        false,
        "检查来源策略、引用完整性和禁用内容后重新执行研究任务",
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        CitationLocation, MissingField, ResearchFact, ResearchSubject, ResearchValue,
        ResearchValueKind, WebCitation, WebSource,
    };
    use chrono::TimeZone;

    fn context<'a>(
        citations: &'a [WebCitation],
        sources: &'a [WebSource],
    ) -> ValidationContext<'a> {
        static KEYS: std::sync::LazyLock<Vec<String>> =
            std::sync::LazyLock::new(|| vec!["home_injuries".to_string()]);
        static POLICY: std::sync::LazyLock<SourcePolicy> =
            std::sync::LazyLock::new(|| SourcePolicy {
                allowed_domains: vec!["example.com".to_string()],
                blocked_domains: vec!["predictions.example".to_string()],
                prohibited_fact_keys: vec!["prediction".to_string(), "odds".to_string()],
                prohibited_content_terms: vec!["betting tip".to_string(), "盘口".to_string()],
                https_only: true,
            });
        ValidationContext {
            match_key: "match-1",
            schema_version: "football.p4-research-output.v2",
            data_cutoff_at: Utc.with_ymd_and_hms(2026, 7, 14, 10, 0, 0).unwrap(),
            requested_fact_keys: &KEYS,
            source_policy: &POLICY,
            citations,
            sources,
        }
    }

    fn valid_output() -> ResearchOutput {
        ResearchOutput {
            schema_version: "football.p4-research-output.v2".to_string(),
            match_key: "match-1".to_string(),
            data_cutoff_at: Utc.with_ymd_and_hms(2026, 7, 14, 10, 0, 0).unwrap(),
            facts: vec![ResearchFact {
                fact_key: "home_injuries.player_a.1".to_string(),
                field_key: "home_injuries".to_string(),
                subject: ResearchSubject {
                    entity_type: "team".to_string(),
                    name: "Home".to_string(),
                    external_id: None,
                },
                value: ResearchValue {
                    kind: ResearchValueKind::StringList,
                    text: None,
                    number: None,
                    integer: None,
                    boolean: None,
                    strings: vec!["Player A unavailable".to_string()],
                },
                verification_state: "CONFIRMED".to_string(),
                source_urls: vec!["https://example.com/team-news".to_string()],
                published_at: Some(Utc.with_ymd_and_hms(2026, 7, 14, 8, 0, 0).unwrap()),
                observed_at: None,
                effective_at: None,
                timezone: Some("UTC".to_string()),
            }],
            missing_fields: Vec::<MissingField>::new(),
        }
    }

    #[test]
    fn accepts_cited_fact_before_cutoff() {
        let citations = vec![WebCitation {
            url: "https://example.com/team-news".to_string(),
            title: "Team news".to_string(),
            domain: "example.com".to_string(),
            location: CitationLocation {
                output_index: 0,
                start_index: Some(10),
                end_index: Some(20),
            },
        }];
        validate_research_output(&valid_output(), &context(&citations, &[])).expect("valid");
    }

    #[test]
    fn rejects_uncited_source_and_post_cutoff_fact() {
        let mut output = valid_output();
        output.facts[0].published_at = Some(Utc.with_ymd_and_hms(2026, 7, 14, 11, 0, 0).unwrap());
        let error = validate_research_output(&output, &context(&[], &[])).expect_err("invalid");
        assert_eq!(error.category, GatewayErrorCategory::SchemaValidation);

        output.facts[0].published_at = None;
        let error = validate_research_output(&output, &context(&[], &[])).expect_err("invalid");
        assert_eq!(error.category, GatewayErrorCategory::SourcePolicy);
    }

    #[test]
    fn rejects_unknown_entity_type_and_empty_string_values() {
        let citations = vec![WebCitation {
            url: "https://example.com/team-news".to_string(),
            title: "Team news".to_string(),
            domain: "example.com".to_string(),
            location: CitationLocation {
                output_index: 0,
                start_index: Some(0),
                end_index: Some(5),
            },
        }];
        let mut output = valid_output();
        output.facts[0].subject.entity_type = "prediction".to_string();
        assert_eq!(
            validate_research_output(&output, &context(&citations, &[]))
                .expect_err("unknown entity type")
                .category,
            GatewayErrorCategory::SchemaValidation
        );

        output.facts[0].subject.entity_type = "team".to_string();
        output.facts[0].value = ResearchValue {
            kind: ResearchValueKind::String,
            text: Some("   ".to_string()),
            number: None,
            integer: None,
            boolean: None,
            strings: vec![],
        };
        assert_eq!(
            validate_research_output(&output, &context(&citations, &[]))
                .expect_err("empty value")
                .category,
            GatewayErrorCategory::SchemaValidation
        );
    }

    #[test]
    fn accepts_multiple_atomic_facts_for_same_requested_field() {
        let citations = vec![
            WebCitation {
                url: "https://example.com/team-news".to_string(),
                title: "Team news".to_string(),
                domain: "example.com".to_string(),
                location: CitationLocation {
                    output_index: 0,
                    start_index: None,
                    end_index: None,
                },
            },
            WebCitation {
                url: "https://example.com/team-news-2".to_string(),
                title: "Team news 2".to_string(),
                domain: "example.com".to_string(),
                location: CitationLocation {
                    output_index: 1,
                    start_index: None,
                    end_index: None,
                },
            },
        ];
        let mut output = valid_output();
        let mut second = output.facts[0].clone();
        second.fact_key = "home_injuries.player_b.1".to_string();
        second.subject.name = "Player B".to_string();
        second.source_urls = vec!["https://example.com/team-news-2".to_string()];
        output.facts.push(second);
        validate_research_output(&output, &context(&citations, &[]))
            .expect("multiple atomic facts");
    }
}
