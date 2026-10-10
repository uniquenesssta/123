use crate::response_fields::{required_string, schema_error};
use crate::{CitationLocation, GatewayError, WebCitation, WebSource};
use serde_json::Value;
use url::Url;

pub(crate) fn parse_citation(
    value: &Value,
    output_index: usize,
) -> Result<WebCitation, GatewayError> {
    let url = required_string(value, "url")?;
    let title = value
        .get("title")
        .and_then(Value::as_str)
        .unwrap_or(&url)
        .to_string();
    let domain = domain_from_url(&url)?;
    Ok(WebCitation {
        url,
        title,
        domain,
        location: CitationLocation {
            output_index,
            start_index: optional_usize(value, "start_index")?,
            end_index: optional_usize(value, "end_index")?,
        },
    })
}

fn optional_usize(value: &Value, key: &str) -> Result<Option<usize>, GatewayError> {
    value
        .get(key)
        .and_then(Value::as_u64)
        .map(|value| {
            usize::try_from(value).map_err(|_| schema_error(format!("引用字段{key}超出usize范围")))
        })
        .transpose()
}

pub(crate) fn parse_source(value: &Value) -> Result<Option<WebSource>, GatewayError> {
    let Some(url) = value.get("url").and_then(Value::as_str) else {
        return Ok(None);
    };
    let domain = domain_from_url(url)?;
    Ok(Some(WebSource {
        url: url.to_string(),
        title: value
            .get("title")
            .and_then(Value::as_str)
            .map(ToString::to_string),
        domain,
    }))
}

fn domain_from_url(value: &str) -> Result<String, GatewayError> {
    Url::parse(value)
        .ok()
        .and_then(|url| url.host_str().map(ToString::to_string))
        .map(|host| host.trim_start_matches("www.").to_lowercase())
        .ok_or_else(|| schema_error(format!("引用包含无效URL：{value}")))
}

#[cfg(test)]
mod tests {
    use super::*;

    use serde_json::json;

    #[test]
    fn wire_citations_keep_original_title_domain_location_and_optional_offset_types() {
        let value = json!({"url":"https://WWW.Example.com/news","start_index":-1,"end_index":"4"});
        let citation = parse_citation(&value, 7).expect("wire citation");
        assert_eq!(citation.url, "https://WWW.Example.com/news");
        assert_eq!(citation.title, citation.url);
        assert_eq!(citation.domain, "example.com");
        assert_eq!(citation.location.output_index, 7);
        assert_eq!(citation.location.start_index, None);
        assert_eq!(citation.location.end_index, None);
        let value =
            json!({"url":"https://example.com/news","start_index":usize::MAX,"end_index":0});
        let citation = parse_citation(&value, 0).expect("usize bounds");
        assert_eq!(citation.location.start_index, Some(usize::MAX));
        assert_eq!(citation.location.end_index, Some(0));
    }

    #[test]
    fn wire_sources_skip_missing_urls_and_reject_invalid_hosts_without_policy_checks() {
        for value in [json!({}), json!({"url":null}), json!({"url":7})] {
            assert!(parse_source(&value).unwrap().is_none());
        }
        let source = parse_source(&json!({"url":"http://example.com/news","title":""}))
            .unwrap()
            .unwrap();
        assert_eq!(source.domain, "example.com");
        assert_eq!(source.title.as_deref(), Some(""));
        for url in ["not a URL", "mailto:user@example.com"] {
            assert_eq!(
                parse_source(&json!({"url":url}))
                    .expect_err("host")
                    .user_message,
                format!("引用包含无效URL：{url}")
            );
        }
    }

    #[test]
    fn wire_required_strings_preserve_original_errors_and_empty_url_rejection() {
        let error = parse_citation(&json!({"url":null}), 0).expect_err("required string");
        assert_eq!(error.user_message, "Responses API响应缺少字符串字段：url");
        assert_eq!(
            error.recovery.action,
            "保留原始响应并检查OpenAI响应结构、Prompt和Schema版本"
        );
        assert_eq!(
            parse_citation(&json!({"url":""}), 0)
                .expect_err("empty URL")
                .user_message,
            "引用包含无效URL："
        );
    }
}
