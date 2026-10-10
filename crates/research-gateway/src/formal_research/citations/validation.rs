use crate::validation::validate_url;
use crate::validation_error::source_error;
use crate::{GatewayError, SourcePolicy, WebCitation, WebSource};
use std::collections::{BTreeMap, BTreeSet};

pub(crate) fn build_source_index(
    citations: &[WebCitation],
    sources: &[WebSource],
    policy: &SourcePolicy,
) -> Result<BTreeMap<String, String>, GatewayError> {
    let mut index = BTreeMap::new();
    for source in sources {
        let normalized = validate_url(&source.url, policy)?;
        index.insert(normalized, source.domain.to_lowercase());
    }
    for citation in citations {
        let normalized = validate_url(&citation.url, policy)?;
        index.insert(normalized, citation.domain.to_lowercase());
    }
    Ok(index)
}

pub(crate) fn validate_fact_sources(
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

#[cfg(test)]
mod tests {
    use super::*;

    use crate::{CitationLocation, GatewayErrorCategory};

    fn policy() -> SourcePolicy {
        SourcePolicy {
            allowed_domains: vec!["example.com".to_string()],
            blocked_domains: vec![],
            prohibited_fact_keys: vec![],
            prohibited_content_terms: vec![],
            https_only: true,
        }
    }

    #[test]
    fn source_index_normalizes_fragments_and_preserves_original_citation_precedence() {
        let sources = vec![WebSource {
            url: "https://example.com/news#search".to_string(),
            title: None,
            domain: "SOURCE.EXAMPLE".to_string(),
        }];
        let citations = vec![WebCitation {
            url: "https://example.com/news#citation".to_string(),
            title: "News".to_string(),
            domain: "CITATION.EXAMPLE".to_string(),
            location: CitationLocation {
                output_index: 0,
                start_index: None,
                end_index: None,
            },
        }];
        let index = build_source_index(&citations, &sources, &policy()).expect("index");
        assert_eq!(index.len(), 1);
        assert_eq!(index["https://example.com/news"], "citation.example");
        assert_eq!(sources[0].url, "https://example.com/news#search");
        assert_eq!(citations[0].url, "https://example.com/news#citation");
    }

    #[test]
    fn fact_sources_keep_required_states_membership_and_normalized_duplicate_rejection() {
        let policy = policy();
        let index = BTreeMap::from([(
            "https://example.com/news".to_string(),
            "example.com".to_string(),
        )]);
        for state in ["NOT_FOUND", "NOT_APPLICABLE"] {
            validate_fact_sources(state, &[], &index, &policy).expect("no source required");
        }
        for state in ["CONFIRMED", "PROBABLE", "CONFLICT", "STALE"] {
            assert_eq!(
                validate_fact_sources(state, &[], &index, &policy)
                    .expect_err("required")
                    .user_message,
                "有事实结论的字段缺少Web Search来源"
            );
        }
        let urls = vec!["https://example.com/news#one".to_string()];
        validate_fact_sources("CONFIRMED", &urls, &index, &policy).expect("indexed normalized URL");
        let duplicate = vec![urls[0].clone(), "https://example.com/news#two".to_string()];
        assert_eq!(
            validate_fact_sources("CONFIRMED", &duplicate, &index, &policy)
                .expect_err("duplicate")
                .user_message,
            "同一事实包含重复来源URL"
        );
        let error = validate_fact_sources(
            "CONFIRMED",
            &["https://example.com/other".to_string()],
            &index,
            &policy,
        )
        .expect_err("not indexed");
        assert_eq!(error.category, GatewayErrorCategory::SourcePolicy);
        assert!(error.user_message.contains("完整搜索来源清单"));
    }
}
