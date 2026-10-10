use crate::formal_research::citations::{build_source_index, validate_fact_sources};
use crate::validation::validate_time;
use crate::validation_error::{schema_error, source_error};
use crate::{
    GatewayError, ResearchOutput, ResearchValue, ResearchValueKind, SourcePolicy, WebCitation,
    WebSource,
};
use chrono::{DateTime, Utc};
use std::collections::BTreeSet;

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
    let source_index =
        build_source_index(context.citations, context.sources, context.source_policy)?;
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::GatewayErrorCategory;
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

    fn response_fixture() -> Vec<WebCitation> {
        vec![WebCitation {
            url: "https://example.com/team-news".to_string(),
            title: "News".to_string(),
            domain: "example.com".to_string(),
            location: CitationLocation {
                output_index: 0,
                start_index: None,
                end_index: None,
            },
        }]
    }

    #[test]
    fn identity_errors_keep_original_match_schema_and_cutoff_priority() {
        let mut output = valid_output();
        output.match_key = "other".to_string();
        output.schema_version = "other".to_string();
        output.data_cutoff_at += chrono::Duration::nanoseconds(1);
        let ctx = context(&[], &[]);
        assert_eq!(
            validate_research_output(&output, &ctx)
                .expect_err("match")
                .user_message,
            "联网输出的比赛键与研究任务不一致"
        );
        output.match_key = ctx.match_key.to_string();
        assert_eq!(
            validate_research_output(&output, &ctx)
                .expect_err("schema")
                .user_message,
            "联网输出的Schema版本与研究任务不一致"
        );
        output.schema_version = ctx.schema_version.to_string();
        assert_eq!(
            validate_research_output(&output, &ctx)
                .expect_err("cutoff")
                .user_message,
            "联网输出的数据截止时间与研究任务不一致"
        );
    }

    #[test]
    fn atomic_fact_limit_is_256_and_shared_fields_do_not_collapse_facts() {
        let citations = response_fixture();
        let mut output = valid_output();
        let template = output.facts[0].clone();
        output.facts = (0..256)
            .map(|index| {
                let mut fact = template.clone();
                fact.fact_key = format!("fact.{index}");
                fact
            })
            .collect();
        validate_research_output(&output, &context(&citations, &[])).expect("256 atomic facts");
        let mut last = template;
        last.fact_key = "fact.256".to_string();
        output.facts.push(last);
        assert_eq!(
            validate_research_output(&output, &context(&citations, &[]))
                .expect_err("257")
                .user_message,
            "联网输出的原子事实数量超过256条上限"
        );
    }

    #[test]
    fn fact_keys_keep_ascii_length_uniqueness_and_requested_field_identity() {
        let citations = response_fixture();
        let ctx = context(&citations, &[]);
        for key in ["", " ", "invalid key", "中文", &"a".repeat(121)] {
            let mut output = valid_output();
            output.facts[0].fact_key = key.to_string();
            assert_eq!(
                validate_research_output(&output, &ctx)
                    .expect_err("fact key")
                    .category,
                GatewayErrorCategory::SchemaValidation
            );
        }
        let mut output = valid_output();
        output.facts[0].fact_key = "a".repeat(120);
        validate_research_output(&output, &ctx).expect("120 ASCII");
        output.facts[0].fact_key = "fact._:-1".to_string();
        validate_research_output(&output, &ctx).expect("allowed punctuation");
        output.facts.push(output.facts[0].clone());
        assert!(validate_research_output(&output, &ctx)
            .expect_err("duplicate")
            .user_message
            .contains("重复fact_key"));
        output.facts.pop();
        output.facts[0].field_key = " home_injuries".to_string();
        assert!(validate_research_output(&output, &ctx)
            .expect_err("raw field identity")
            .user_message
            .contains("未请求字段"));
    }

    #[test]
    fn missing_fields_require_complete_nonconflicting_unique_coverage() {
        let mut output = valid_output();
        output.facts.clear();
        let ctx = context(&[], &[]);
        assert!(validate_research_output(&output, &ctx)
            .expect_err("coverage")
            .user_message
            .contains("明确缺失状态"));
        for state in ["NOT_FOUND", "STALE", "NOT_APPLICABLE"] {
            output.missing_fields = vec![MissingField {
                field_key: "home_injuries".to_string(),
                verification_state: state.to_string(),
            }];
            validate_research_output(&output, &ctx).expect("valid missing state");
        }
        output.missing_fields[0].verification_state = "CONFIRMED".to_string();
        assert!(validate_research_output(&output, &ctx)
            .expect_err("missing state")
            .user_message
            .contains("无效状态"));
        output.missing_fields[0].verification_state = "NOT_FOUND".to_string();
        output.missing_fields.push(output.missing_fields[0].clone());
        assert!(validate_research_output(&output, &ctx)
            .expect_err("duplicate missing")
            .user_message
            .contains("重复缺失"));
        output.missing_fields.pop();
        let citations = response_fixture();
        output.facts = valid_output().facts;
        assert!(validate_research_output(&output, &context(&citations, &[]))
            .expect_err("fact plus missing")
            .user_message
            .contains("同时出现在事实"));
    }

    #[test]
    fn subjects_preserve_entity_enum_and_unicode_character_limit() {
        for entity in [
            "match",
            "competition",
            "venue",
            "team",
            "player",
            "coach",
            "official",
        ] {
            validate_subject(entity, &"中".repeat(200)).expect("known entity and 200 chars");
        }
        for name in ["", " \t", &"中".repeat(201)] {
            assert_eq!(
                validate_subject("team", name)
                    .expect_err("name")
                    .user_message,
                "事实主体名称不能为空且不能超过200个字符"
            );
        }
        assert_eq!(
            validate_subject("TEAM", "Home")
                .expect_err("raw enum")
                .user_message,
            "未知事实实体类型：TEAM"
        );
    }

    #[test]
    fn value_kinds_require_one_matching_payload_and_finite_numbers() {
        let empty = ResearchValue {
            kind: ResearchValueKind::Null,
            text: None,
            number: None,
            integer: None,
            boolean: None,
            strings: vec![],
        };
        let mut values = vec![empty.clone()];
        let mut value = empty.clone();
        value.kind = ResearchValueKind::String;
        value.text = Some("news".to_string());
        values.push(value);
        let mut value = empty.clone();
        value.kind = ResearchValueKind::Number;
        value.number = Some(0.0);
        values.push(value);
        let mut value = empty.clone();
        value.kind = ResearchValueKind::Integer;
        value.integer = Some(i64::MIN);
        values.push(value);
        let mut value = empty.clone();
        value.kind = ResearchValueKind::Boolean;
        value.boolean = Some(false);
        values.push(value);
        let mut value = empty;
        value.kind = ResearchValueKind::StringList;
        value.strings = vec!["news".to_string()];
        values.push(value);
        for value in &values {
            validate_value(value, &[]).expect("matching payload");
        }
        for mut value in values {
            value.integer = Some(1);
            value.boolean = Some(true);
            assert_eq!(
                validate_value(&value, &[])
                    .expect_err("mixed payload")
                    .user_message,
                "事实值kind与实际载荷不一致"
            );
        }
        for number in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            let value = ResearchValue {
                kind: ResearchValueKind::Number,
                text: None,
                number: Some(number),
                integer: None,
                boolean: None,
                strings: vec![],
            };
            assert!(validate_value(&value, &[]).is_err());
        }
    }

    #[test]
    fn verification_and_prohibited_content_keep_existing_policy_mapping() {
        for state in [
            "CONFIRMED",
            "PROBABLE",
            "CONFLICT",
            "NOT_FOUND",
            "STALE",
            "NOT_APPLICABLE",
        ] {
            validate_verification_state(state).expect("known state");
        }
        assert_eq!(
            validate_verification_state("confirmed")
                .expect_err("raw enum")
                .category,
            GatewayErrorCategory::SchemaValidation
        );
        let mut value = valid_output().facts.remove(0).value;
        value.strings = vec!["BETTING".to_string(), "TIP".to_string()];
        let error = validate_value(&value, &["betting tip".to_string()])
            .expect_err("joined forbidden content");
        assert_eq!(error.category, GatewayErrorCategory::SourcePolicy);
        assert!(!error.recovery.retryable);
        assert_eq!(
            error.recovery.action,
            "检查来源策略、引用完整性和禁用内容后重新执行研究任务"
        );
    }

    #[test]
    fn all_three_fact_times_still_delegate_to_the_same_cutoff_policy() {
        let citations = response_fixture();
        let ctx = context(&citations, &[]);
        for field in ["published_at", "observed_at", "effective_at"] {
            for offset in [-1, 0, 1] {
                let mut output = valid_output();
                let time = Some(ctx.data_cutoff_at + chrono::Duration::nanoseconds(offset));
                match field {
                    "published_at" => output.facts[0].published_at = time,
                    "observed_at" => output.facts[0].observed_at = time,
                    _ => output.facts[0].effective_at = time,
                }
                let result = validate_research_output(&output, &ctx);
                if offset <= 0 {
                    result.expect("at or before cutoff");
                } else {
                    assert_eq!(
                        result.expect_err("after cutoff").user_message,
                        format!("{field}晚于赛前数据截止时间，不能进入当前研究结果")
                    );
                }
            }
        }
    }
}
