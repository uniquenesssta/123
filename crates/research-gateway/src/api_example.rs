use crate::{ApiProtocol, GatewayError, GatewayErrorCategory, TokenLimitField};
use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};
use url::Url;

const API_KEY_PLACEHOLDER: &str = "YOUR_API_KEY";

#[derive(Clone, Serialize, Deserialize, PartialEq)]
pub struct ApiExampleCandidate {
    pub protocol: ApiProtocol,
    pub endpoint_url: String,
    pub api_base_url: String,
    pub model_id: Option<String>,
    pub max_output_tokens: Option<u32>,
    pub token_limit_field: TokenLimitField,
    pub api_key: Option<String>,
    pub has_authorization_header: bool,
    pub sanitized_example: String,
    pub formal_research_candidate: bool,
    pub warnings: Vec<String>,
}

#[derive(Clone, Serialize, Deserialize, PartialEq)]
pub struct ApiExampleParseResult {
    pub selected: ApiExampleCandidate,
    pub candidates: Vec<ApiExampleCandidate>,
}

pub fn parse_api_example(
    input: &str,
    preferred_protocol: Option<ApiProtocol>,
) -> Result<ApiExampleParseResult, GatewayError> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err(example_error("API Example不能为空"));
    }
    if trimmed.len() > 200_000 {
        return Err(example_error("API Example过长，最多允许200000个字符"));
    }

    let mut candidates = Vec::new();
    let mut first_error = None;
    for command in extract_curl_commands(trimmed) {
        match parse_curl_candidate(&command) {
            Ok(candidate) => candidates.push(candidate),
            Err(error) => {
                if first_error.is_none() {
                    first_error = Some(error);
                }
            }
        }
    }

    if candidates.is_empty() {
        if let Ok(value) = serde_json::from_str::<Value>(trimmed) {
            candidates.push(parse_json_candidate(&value)?);
        }
    }

    if candidates.is_empty() {
        return Err(first_error.unwrap_or_else(|| {
            example_error("没有识别到可用的curl或JSON API Example；请至少包含请求URL和JSON请求体")
        }));
    }

    deduplicate_candidates(&mut candidates);
    let selected = preferred_protocol
        .and_then(|protocol| {
            candidates
                .iter()
                .find(|candidate| candidate.protocol == protocol)
                .cloned()
        })
        .or_else(|| {
            candidates
                .iter()
                .find(|candidate| candidate.protocol == ApiProtocol::Responses)
                .cloned()
        })
        .unwrap_or_else(|| candidates[0].clone());

    Ok(ApiExampleParseResult {
        selected,
        candidates,
    })
}

fn parse_curl_candidate(command: &str) -> Result<ApiExampleCandidate, GatewayError> {
    let tokens = shell_tokens(command)?;
    if tokens.is_empty()
        || !matches!(
            tokens[0].as_str(),
            "curl" | "curl.exe" | "CURL" | "CURL.EXE"
        )
    {
        return Err(example_error("API Example不是有效的curl命令"));
    }

    let mut endpoint_url = None;
    let mut headers = Vec::<String>::new();
    let mut body_text = None;
    let mut index = 1usize;
    while index < tokens.len() {
        let token = &tokens[index];
        match token.as_str() {
            "-H" | "--header" => {
                index += 1;
                let value = tokens
                    .get(index)
                    .ok_or_else(|| example_error("curl的Header参数缺少值"))?;
                headers.push(value.clone());
            }
            "-d" | "--data" | "--data-raw" | "--data-binary" => {
                index += 1;
                let value = tokens
                    .get(index)
                    .ok_or_else(|| example_error("curl的请求体参数缺少值"))?;
                body_text = Some(value.clone());
            }
            "--url" => {
                index += 1;
                endpoint_url = tokens.get(index).cloned();
            }
            "-X" | "--request" => {
                index += 1;
            }
            _ if token.starts_with("http://") || token.starts_with("https://") => {
                endpoint_url = Some(token.clone());
            }
            _ => {}
        }
        index += 1;
    }

    let endpoint_url = endpoint_url.ok_or_else(|| example_error("curl中缺少HTTP请求URL"))?;
    let body_text = body_text.ok_or_else(|| example_error("curl中缺少JSON请求体"))?;
    let body = serde_json::from_str::<Value>(&body_text)
        .map_err(|error| example_error(format!("curl请求体不是有效JSON：{error}")))?;
    let authorization = headers
        .iter()
        .find_map(|header| split_header(header, "authorization"));
    let api_key = authorization
        .as_deref()
        .and_then(extract_bearer_token)
        .filter(|value| !is_placeholder_key(value));
    build_candidate(&endpoint_url, &body, authorization.is_some(), api_key)
}

fn parse_json_candidate(value: &Value) -> Result<ApiExampleCandidate, GatewayError> {
    let object = value
        .as_object()
        .ok_or_else(|| example_error("JSON API Example必须是对象"))?;
    let endpoint_url = object
        .get("url")
        .or_else(|| object.get("endpoint"))
        .and_then(Value::as_str)
        .ok_or_else(|| example_error("JSON API Example缺少url或endpoint"))?;
    let body = object
        .get("body")
        .or_else(|| object.get("data"))
        .ok_or_else(|| example_error("JSON API Example缺少body或data"))?;
    let headers = object.get("headers").and_then(Value::as_object);
    let authorization = headers.and_then(|headers| {
        headers.iter().find_map(|(key, value)| {
            key.eq_ignore_ascii_case("authorization")
                .then(|| value.as_str())
                .flatten()
        })
    });
    let api_key = authorization
        .and_then(extract_bearer_token)
        .filter(|value| !is_placeholder_key(value));
    build_candidate(endpoint_url, body, authorization.is_some(), api_key)
}

fn build_candidate(
    endpoint_url: &str,
    body: &Value,
    has_authorization_header: bool,
    api_key: Option<String>,
) -> Result<ApiExampleCandidate, GatewayError> {
    let endpoint = validate_endpoint(endpoint_url)?;
    let protocol = detect_protocol(&endpoint, body)?;
    let api_base_url = derive_api_base_url(endpoint.as_str(), protocol)?;
    let model_id = body
        .get("model")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToString::to_string);
    let (token_limit_field, max_output_tokens) = detect_token_limit(body, protocol);
    let sanitized_body = sanitize_body(body);
    let sanitized_example = canonical_curl(endpoint.as_str(), &sanitized_body);
    let mut warnings = Vec::new();
    if model_id.is_none() {
        warnings.push("请求体没有model字段；解析后不会替换现有模型ID".to_string());
    }
    if !has_authorization_header {
        warnings.push("示例没有Authorization Header；保存前仍需单独填写API Key".to_string());
    }
    if protocol == ApiProtocol::ChatCompletions {
        warnings.push(
            "Chat Completions可用于连接测试和API协作；P4正式联网研究仍要求Responses协议与可验证来源元数据"
                .to_string(),
        );
    } else {
        warnings.push(
            "Responses端点已识别；正式联网研究还要求该兼容服务支持json_schema、tools和Web Search来源元数据"
                .to_string(),
        );
    }

    Ok(ApiExampleCandidate {
        protocol,
        endpoint_url: endpoint.to_string().trim_end_matches('/').to_string(),
        api_base_url,
        model_id,
        max_output_tokens,
        token_limit_field,
        api_key,
        has_authorization_header,
        sanitized_example,
        formal_research_candidate: protocol == ApiProtocol::Responses,
        warnings,
    })
}

fn validate_endpoint(value: &str) -> Result<Url, GatewayError> {
    let url = Url::parse(value.trim()).map_err(|_| example_error("API Example中的请求URL无效"))?;
    let is_loopback = url
        .host_str()
        .is_some_and(|host| matches!(host, "127.0.0.1" | "localhost" | "::1"));
    if url.scheme() != "https" && !(url.scheme() == "http" && is_loopback) {
        return Err(example_error(
            "API Example请求URL必须使用HTTPS；仅本机回环地址允许HTTP",
        ));
    }
    if !url.username().is_empty() || url.password().is_some() {
        return Err(example_error("API Example请求URL不能内嵌用户名或密码"));
    }
    if url.query().is_some() || url.fragment().is_some() {
        return Err(example_error("API Example请求URL不能包含查询参数或片段"));
    }
    Ok(url)
}

fn detect_protocol(url: &Url, body: &Value) -> Result<ApiProtocol, GatewayError> {
    let path = url.path().trim_end_matches('/');
    let path_protocol = if path.ends_with("/responses") {
        Some(ApiProtocol::Responses)
    } else if path.ends_with("/chat/completions") {
        Some(ApiProtocol::ChatCompletions)
    } else {
        None
    };
    let body_protocol = if body.get("input").is_some() {
        Some(ApiProtocol::Responses)
    } else if body.get("messages").is_some() {
        Some(ApiProtocol::ChatCompletions)
    } else {
        None
    };
    match (path_protocol, body_protocol) {
        (Some(path), Some(body)) if path != body => {
            Err(example_error("API Example的请求URL与JSON请求体协议不一致"))
        }
        (Some(protocol), _) | (_, Some(protocol)) => Ok(protocol),
        (None, None) => Err(example_error(
            "无法判断API协议；请求URL需以/responses或/chat/completions结尾",
        )),
    }
}

fn derive_api_base_url(endpoint_url: &str, protocol: ApiProtocol) -> Result<String, GatewayError> {
    let suffix = protocol.endpoint_suffix();
    let normalized = endpoint_url.trim_end_matches('/');
    let base = normalized
        .strip_suffix(suffix)
        .ok_or_else(|| example_error("请求URL与识别出的API协议不一致"))?
        .trim_end_matches('/');
    if base.is_empty() {
        return Err(example_error("无法从请求URL推导API基础地址"));
    }
    Ok(base.to_string())
}

fn detect_token_limit(body: &Value, protocol: ApiProtocol) -> (TokenLimitField, Option<u32>) {
    if let Some(value) = body.get("max_output_tokens").and_then(Value::as_u64) {
        return (TokenLimitField::MaxOutputTokens, u32::try_from(value).ok());
    }
    if let Some(value) = body.get("max_tokens").and_then(Value::as_u64) {
        return (TokenLimitField::MaxTokens, u32::try_from(value).ok());
    }
    (protocol.default_token_limit_field(), None)
}

fn sanitize_body(body: &Value) -> Value {
    match body {
        Value::Object(object) => {
            let sanitized = object
                .iter()
                .map(|(key, value)| (key.clone(), sanitize_body(value)))
                .collect::<Map<String, Value>>();
            Value::Object(sanitized)
        }
        Value::Array(values) => Value::Array(values.iter().map(sanitize_body).collect()),
        Value::String(value) if looks_like_secret(value) => {
            Value::String(API_KEY_PLACEHOLDER.to_string())
        }
        _ => body.clone(),
    }
}

fn canonical_curl(endpoint_url: &str, body: &Value) -> String {
    let pretty = serde_json::to_string_pretty(body).unwrap_or_else(|_| json!({}).to_string());
    format!(
        "curl {endpoint_url} \\\n  -H \"Content-Type: application/json\" \\\n  -H \"Authorization: Bearer {API_KEY_PLACEHOLDER}\" \\\n  -d '{pretty}'"
    )
}

fn split_header(header: &str, name: &str) -> Option<String> {
    let (header_name, value) = header.split_once(':')?;
    header_name
        .trim()
        .eq_ignore_ascii_case(name)
        .then(|| value.trim().to_string())
}

fn extract_bearer_token(value: &str) -> Option<String> {
    let mut parts = value.split_whitespace();
    let scheme = parts.next()?;
    let token = parts.next()?;
    if parts.next().is_some() || !scheme.eq_ignore_ascii_case("bearer") || token.is_empty() {
        return None;
    }
    Some(token.to_string())
}

fn is_placeholder_key(value: &str) -> bool {
    let normalized = value.trim().to_ascii_uppercase();
    normalized.is_empty()
        || normalized.contains("YOUR_API_KEY")
        || normalized.contains("API_KEY_HERE")
        || normalized.contains("REPLACE_ME")
        || normalized.starts_with('<')
        || normalized.starts_with("${")
        || normalized == "XXX"
        || normalized == "TOKEN"
}

fn looks_like_secret(value: &str) -> bool {
    let trimmed = value.trim();
    !is_placeholder_key(trimmed)
        && trimmed.len() >= 20
        && !trimmed.chars().any(char::is_whitespace)
        && (trimmed.starts_with("sk-")
            || trimmed.starts_with("sk_")
            || trimmed.starts_with("Bearer "))
}

fn extract_curl_commands(input: &str) -> Vec<String> {
    let mut commands = Vec::new();
    let mut search_from = 0usize;
    while let Some(relative) = input[search_from..].find("curl ") {
        let start = search_from + relative;
        if start > 0 {
            let previous = input[..start].chars().next_back();
            if previous.is_some_and(|value| value.is_ascii_alphanumeric() || value == '_') {
                search_from = start + 5;
                continue;
            }
        }
        let command = scan_shell_command(&input[start..]);
        if !command.trim().is_empty() {
            commands.push(command);
        }
        search_from = start + 5;
    }
    commands
}

fn scan_shell_command(input: &str) -> String {
    let mut quote = None::<char>;
    let mut escaped = false;
    let mut previous_non_whitespace = None::<char>;
    let mut end = input.len();
    for (index, character) in input.char_indices() {
        if escaped {
            escaped = false;
            if !character.is_whitespace() {
                previous_non_whitespace = Some(character);
            }
            continue;
        }
        match quote {
            Some('\'') => {
                if character == '\'' {
                    quote = None;
                }
            }
            Some('"') => {
                if character == '\\' {
                    escaped = true;
                } else if character == '"' {
                    quote = None;
                }
            }
            None => {
                if character == '\'' || character == '"' {
                    quote = Some(character);
                } else if character == '\\' {
                    escaped = true;
                } else if character == '\n' {
                    if previous_non_whitespace != Some('\\') {
                        end = index;
                        break;
                    }
                    previous_non_whitespace = None;
                    continue;
                }
            }
            _ => {}
        }
        if !character.is_whitespace() {
            previous_non_whitespace = Some(character);
        }
    }
    input[..end].to_string()
}

fn shell_tokens(input: &str) -> Result<Vec<String>, GatewayError> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut quote = None::<char>;
    let mut escaped = false;
    let mut chars = input.chars().peekable();

    while let Some(character) = chars.next() {
        if escaped {
            if character != '\n' && character != '\r' {
                current.push(character);
            }
            escaped = false;
            continue;
        }
        match quote {
            Some('\'') => {
                if character == '\'' {
                    quote = None;
                } else {
                    current.push(character);
                }
            }
            Some('"') => {
                if character == '"' {
                    quote = None;
                } else if character == '\\' {
                    escaped = true;
                } else {
                    current.push(character);
                }
            }
            None => {
                if character == '\'' || character == '"' {
                    quote = Some(character);
                } else if character == '\\' {
                    if chars.peek().is_some_and(|next| *next == '\r') {
                        chars.next();
                    }
                    if chars.peek().is_some_and(|next| *next == '\n') {
                        chars.next();
                    } else {
                        escaped = true;
                    }
                } else if character.is_whitespace() {
                    if !current.is_empty() {
                        tokens.push(std::mem::take(&mut current));
                    }
                } else {
                    current.push(character);
                }
            }
            _ => {}
        }
    }

    if quote.is_some() {
        return Err(example_error("curl命令包含未闭合的引号"));
    }
    if escaped {
        current.push('\\');
    }
    if !current.is_empty() {
        tokens.push(current);
    }
    Ok(tokens)
}

fn deduplicate_candidates(candidates: &mut Vec<ApiExampleCandidate>) {
    let mut seen = std::collections::BTreeSet::new();
    candidates
        .retain(|candidate| seen.insert((candidate.protocol, candidate.endpoint_url.clone())));
}

fn example_error(message: impl Into<String>) -> GatewayError {
    GatewayError::new(
        GatewayErrorCategory::InvalidConfiguration,
        message,
        false,
        "粘贴完整curl示例，或提供包含url、headers和body的JSON对象",
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    const MULTI_EXAMPLE: &str = r#"
### Chat
curl https://api.gptsapi.net/v1/chat/completions \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer YOUR_API_KEY" \
  -d '{"model":"gpt-5.6-sol","messages":[{"role":"user","content":"你好"}],"max_tokens":1000}'

### Responses
curl https://api.gptsapi.net/v1/responses \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer YOUR_API_KEY" \
  -d '{"model":"gpt-5.6-sol","input":[{"role":"user","content":[{"type":"input_text","text":"你好"}]}],"max_tokens":1000}'
"#;

    #[test]
    fn prefers_responses_when_markdown_contains_two_examples() {
        let parsed = parse_api_example(MULTI_EXAMPLE, None).expect("解析API Example");
        assert_eq!(parsed.candidates.len(), 2);
        assert_eq!(parsed.selected.protocol, ApiProtocol::Responses);
        assert_eq!(parsed.selected.api_base_url, "https://api.gptsapi.net/v1");
        assert_eq!(parsed.selected.model_id.as_deref(), Some("gpt-5.6-sol"));
        assert_eq!(
            parsed.selected.token_limit_field,
            TokenLimitField::MaxTokens
        );
        assert_eq!(parsed.selected.max_output_tokens, Some(1000));
        assert!(parsed.selected.api_key.is_none());
    }

    #[test]
    fn can_select_chat_candidate_explicitly() {
        let parsed = parse_api_example(MULTI_EXAMPLE, Some(ApiProtocol::ChatCompletions))
            .expect("解析Chat API Example");
        assert_eq!(parsed.selected.protocol, ApiProtocol::ChatCompletions);
        assert_eq!(
            parsed.selected.endpoint_url,
            "https://api.gptsapi.net/v1/chat/completions"
        );
        assert!(!parsed.selected.formal_research_candidate);
    }

    #[test]
    fn extracts_real_key_but_never_keeps_it_in_sanitized_example() {
        let example = MULTI_EXAMPLE.replace("YOUR_API_KEY", "sk-test-secret-value-1234567890");
        let parsed =
            parse_api_example(&example, Some(ApiProtocol::Responses)).expect("解析真实密钥示例");
        assert_eq!(
            parsed.selected.api_key.as_deref(),
            Some("sk-test-secret-value-1234567890")
        );
        assert!(!parsed
            .selected
            .sanitized_example
            .contains("sk-test-secret-value-1234567890"));
        assert!(parsed
            .selected
            .sanitized_example
            .contains(API_KEY_PLACEHOLDER));
    }
}
