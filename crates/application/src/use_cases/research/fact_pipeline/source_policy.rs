use super::*;

pub(super) fn build_source_index(
    command: &ProcessResearchEvidenceCommand,
    policy: &SourcePolicyDefinition,
) -> ApplicationResult<BTreeMap<String, SourceReference>> {
    let mut index = BTreeMap::new();
    for source in &command.sources {
        let (normalized, domain) = normalize_source_url(&source.url)?;
        let (tier, rank, independence_key) = classify_source(&domain, policy)?;
        index.insert(
            normalized,
            SourceReference {
                url: source.url.clone(),
                title: source.title.clone().unwrap_or_else(|| domain.clone()),
                domain,
                independence_key,
                tier,
                rank,
            },
        );
    }
    for citation in &command.citations {
        let (normalized, domain) = normalize_source_url(&citation.url)?;
        let (tier, rank, independence_key) = classify_source(&domain, policy)?;
        index.insert(
            normalized,
            SourceReference {
                url: citation.url.clone(),
                title: citation.title.clone(),
                domain,
                independence_key,
                tier,
                rank,
            },
        );
    }
    Ok(index)
}

pub(super) fn classify_source(
    domain: &str,
    policy: &SourcePolicyDefinition,
) -> ApplicationResult<(String, u16, String)> {
    let host = normalize_domain(domain);
    let matched_rule = policy
        .domain_rules
        .iter()
        .filter(|rule| domain_matches(&host, &rule.domain))
        .max_by_key(|rule| normalize_domain(&rule.domain).len());
    let tier = matched_rule
        .map(|rule| rule.tier.as_str())
        .unwrap_or(policy.default_tier.as_str());
    let rank = policy
        .tiers
        .iter()
        .find(|definition| definition.key == tier)
        .map(|definition| definition.rank)
        .ok_or_else(|| ApplicationError::Validation(format!("来源策略引用了未定义等级：{tier}")))?;
    let independence_key = matched_rule
        .map(|rule| normalize_domain(&rule.domain))
        .unwrap_or_else(|| host.clone());
    Ok((tier.to_string(), rank, independence_key))
}

pub(super) fn domain_matches(host: &str, configured: &str) -> bool {
    let configured = normalize_domain(configured);
    host == configured || host.ends_with(&format!(".{configured}"))
}

pub(super) fn normalize_domain(value: &str) -> String {
    value
        .trim()
        .trim_start_matches("*.")
        .trim_start_matches("www.")
        .to_lowercase()
}

pub(super) fn normalize_source_url(value: &str) -> ApplicationResult<(String, String)> {
    let mut url = Url::parse(value)
        .map_err(|_| ApplicationError::Validation(format!("无法规范化来源URL：{value}")))?;
    if url.scheme() != "https" || !url.username().is_empty() || url.password().is_some() {
        return Err(ApplicationError::Validation(format!(
            "来源URL必须使用HTTPS且不能包含用户名或密码：{value}"
        )));
    }
    let domain = url
        .host_str()
        .map(normalize_domain)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| ApplicationError::Validation(format!("来源URL缺少有效域名：{value}")))?;
    url.set_fragment(None);
    Ok((url.to_string(), domain))
}

pub(super) fn normalize_url(value: &str) -> ApplicationResult<String> {
    normalize_source_url(value).map(|(url, _)| url)
}

pub(super) fn built_in_source_policy() -> SourcePolicyVersionDraft {
    SourcePolicyVersionDraft {
        policy_key: SOURCE_POLICY_KEY.to_string(),
        version: SOURCE_POLICY_SEMVER.to_string(),
        competition_profile_id: None,
        definition: serde_json::from_str(include_str!(
            "../../../../../../src-tauri/resources/research/public_source_policy.json"
        ))
        .expect("内置P4来源策略必须有效"),
        metadata: json!({
            "stage": "E",
            "schema_version": P4_SOURCE_POLICY_VERSION,
            "competition_override_supported": true
        }),
    }
}
