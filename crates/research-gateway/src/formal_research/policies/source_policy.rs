use crate::validation_error::source_error;
use crate::{GatewayError, ResearchValue, ResearchValueKind, SourcePolicy};
use url::Url;

pub(crate) fn validate_url(value: &str, policy: &SourcePolicy) -> Result<String, GatewayError> {
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

// Prepared once for one output validation; no cross-request state or configuration mutation.
pub(crate) struct FactContentPolicy {
    prohibited_keys: Vec<String>,
    prohibited_terms: Vec<String>,
}

impl FactContentPolicy {
    pub(crate) fn new(policy: &SourcePolicy) -> Self {
        let prohibited_keys: Vec<String> = policy
            .prohibited_fact_keys
            .iter()
            .map(|value| value.to_lowercase())
            .collect();
        let prohibited_terms: Vec<String> = policy
            .prohibited_content_terms
            .iter()
            .map(|value| value.to_lowercase())
            .collect();
        Self {
            prohibited_keys,
            prohibited_terms,
        }
    }

    pub(crate) fn validate_field_key(&self, field_key: &str) -> Result<(), GatewayError> {
        let normalized_key = field_key.to_lowercase();
        if self
            .prohibited_keys
            .iter()
            .any(|term| normalized_key.contains(term))
        {
            return Err(source_error(format!(
                "字段{}属于禁止进入模型的预测、盘口或推荐内容",
                field_key
            )));
        }
        Ok(())
    }

    pub(crate) fn validate_content(&self, value: &ResearchValue) -> Result<(), GatewayError> {
        let text = match value.kind {
            ResearchValueKind::String => value.text.as_deref().unwrap_or_default().to_lowercase(),
            ResearchValueKind::StringList => value.strings.join(" ").to_lowercase(),
            _ => String::new(),
        };
        if self.prohibited_terms.iter().any(|term| text.contains(term)) {
            return Err(source_error(
                "联网结果包含预测、推荐、盘口或博彩分析内容，已阻止进入事实库",
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::GatewayErrorCategory;

    fn policy() -> SourcePolicy {
        SourcePolicy {
            allowed_domains: vec!["example.com".to_string()],
            blocked_domains: vec![],
            prohibited_fact_keys: vec!["PREDICTION".to_string(), "odds".to_string()],
            prohibited_content_terms: vec!["BETTING TIP".to_string(), "盘口".to_string()],
            https_only: true,
        }
    }

    fn string_value(text: &str) -> ResearchValue {
        ResearchValue {
            kind: ResearchValueKind::String,
            text: Some(text.to_string()),
            number: None,
            integer: None,
            boolean: None,
            strings: vec![],
        }
    }

    #[test]
    fn domain_rules_keep_case_prefix_subdomains_and_dot_boundaries() {
        for configured in ["example.com", " WWW.EXAMPLE.COM ", " *.www.Example.Com "] {
            let mut policy = policy();
            policy.allowed_domains = vec![configured.to_string()];
            for url in [
                "https://EXAMPLE.COM/news",
                "https://www.example.com/news",
                "https://a.b.example.com/news",
            ] {
                validate_url(url, &policy).expect("apex and real subdomains");
            }
            for url in [
                "https://badexample.com/news",
                "https://example.com.evil/news",
            ] {
                assert!(validate_url(url, &policy)
                    .expect_err("domain boundary")
                    .user_message
                    .contains("允许列表"));
            }
        }
    }

    #[test]
    fn source_errors_keep_parse_https_host_block_and_allow_priority() {
        let mut policy = policy();
        policy.blocked_domains = vec!["example.com".to_string()];
        for (url, message) in [
            ("invalid-url", "无效来源URL：invalid-url"),
            (
                "http://example.com/news",
                "来源URL必须使用HTTPS：http://example.com/news",
            ),
            ("https://example.com/news", "来源域名已被禁止：example.com"),
            (
                "https://other.example/news",
                "来源域名不在当前赛事允许列表：other.example",
            ),
        ] {
            let error = validate_url(url, &policy).expect_err("policy rejection");
            assert_eq!(error.user_message, message);
            assert_eq!(error.category, GatewayErrorCategory::SourcePolicy);
            assert!(!error.recovery.retryable);
            assert_eq!(
                error.recovery.action,
                "检查来源策略、引用完整性和禁用内容后重新执行研究任务"
            );
        }
        policy.https_only = false;
        assert_eq!(
            validate_url("mailto:news@example.com", &policy)
                .expect_err("host")
                .user_message,
            "来源URL缺少域名：mailto:news@example.com"
        );
        policy.allowed_domains = vec!["other.example".to_string()];
        assert_eq!(
            validate_url("https://sub.example.com/news", &policy)
                .expect_err("block before allow")
                .user_message,
            "来源域名已被禁止：sub.example.com"
        );
    }

    #[test]
    fn normalized_source_removes_only_fragment_after_original_url_parsing() {
        let policy = policy();
        let original = policy.clone();
        let url = "https://reader:pass@WWW.EXAMPLE.COM:8443/a/../news?q=A%20B#section";
        assert_eq!(
            validate_url(url, &policy).unwrap(),
            "https://reader:pass@www.example.com:8443/news?q=A%20B"
        );
        assert_eq!(
            validate_url("https://example.com/news?q=one#first", &policy).unwrap(),
            "https://example.com/news?q=one"
        );
        assert_ne!(
            validate_url("https://example.com/news?q=one", &policy).unwrap(),
            validate_url("https://example.com/news?q=two", &policy).unwrap()
        );
        assert_eq!(policy, original);
    }

    #[test]
    fn empty_allow_list_and_disabled_https_keep_original_scheme_policy() {
        let mut policy = policy();
        policy.allowed_domains.clear();
        validate_url("https://unlisted.example/news", &policy).expect("no allow restriction");
        policy.https_only = false;
        for url in [
            "http://unlisted.example/news",
            "ftp://unlisted.example/news",
        ] {
            assert_eq!(validate_url(url, &policy).unwrap(), url);
        }
        policy.blocked_domains = vec!["*.unlisted.example".to_string()];
        assert!(validate_url("ftp://sub.unlisted.example/news", &policy)
            .expect_err("block still active")
            .user_message
            .contains("已被禁止"));
    }

    #[test]
    fn content_restrictions_keep_substrings_case_and_joined_string_lists() {
        let policy = policy();
        let original = policy.clone();
        let restrictions = FactContentPolicy::new(&policy);
        for field in ["home_PREDICTION", "ODDS_summary"] {
            assert!(restrictions
                .validate_field_key(field)
                .expect_err("forbidden field")
                .user_message
                .starts_with(&format!("字段{field}")));
        }
        restrictions
            .validate_field_key("home_injuries")
            .expect("requested facts");
        for text in ["latest Betting Tip", "今日盘口资料"] {
            assert_eq!(
                restrictions
                    .validate_content(&string_value(text))
                    .expect_err("forbidden content")
                    .category,
                GatewayErrorCategory::SourcePolicy
            );
        }
        let mut value = string_value("unused payload");
        value.kind = ResearchValueKind::StringList;
        value.text = None;
        value.strings = vec!["BETTING".to_string(), "TIP".to_string()];
        assert!(restrictions.validate_content(&value).is_err());
        restrictions
            .validate_content(&string_value("Team injury update"))
            .expect("allowed content");
        assert_eq!(policy, original);
    }

    #[test]
    fn restrictions_keep_raw_whitespace_and_empty_term_behavior() {
        let mut policy = policy();
        policy.prohibited_fact_keys = vec![" prediction ".to_string()];
        policy.prohibited_content_terms = vec![" betting tip ".to_string()];
        let restrictions = FactContentPolicy::new(&policy);
        restrictions
            .validate_field_key("prediction")
            .expect("no trimming policy terms");
        restrictions
            .validate_content(&string_value("betting tip"))
            .expect("raw term spacing");
        assert!(restrictions
            .validate_content(&string_value("a betting tip b"))
            .is_err());
        policy.prohibited_fact_keys = vec![String::new()];
        policy.prohibited_content_terms = vec![String::new()];
        let restrictions = FactContentPolicy::new(&policy);
        assert!(restrictions.validate_field_key("home_injuries").is_err());
        let mut value = string_value("news");
        assert!(restrictions.validate_content(&value).is_err());
        value.kind = ResearchValueKind::Null;
        value.text = None;
        assert!(restrictions.validate_content(&value).is_err());
    }
}
