use crate::validation_error::{schema_error, source_error};
use crate::{GatewayError, SourcePolicy};
use chrono::{DateTime, Utc};
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

pub(crate) fn validate_time(
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
