//! Shared trust boundary for issue history and runtime traces. No credential reads.
const SECRET_KEYS: &[&str] = &[
    "authorization",
    "proxy-authorization",
    "x-api-key",
    "api_key",
    "api-key",
    "apikey",
    "access_token",
    "access-token",
    "refresh_token",
    "refresh-token",
    "password",
    "secret",
    "credential",
    "credentials",
    "token",
];
const BODY_KEYS: &[&str] = &[
    "prompt",
    "messages",
    "content",
    "body",
    "schema",
    "api_example",
    "api-example",
    "encrypted_content",
    "cookie",
    "set-cookie",
];

pub(crate) fn sensitive_key(key: &str) -> bool {
    let key = key.trim().to_ascii_lowercase();
    SECRET_KEYS.contains(&key.as_str()) || BODY_KEYS.contains(&key.as_str())
}

pub(crate) fn redact_text(value: &str) -> String {
    let mut output = value.to_string();
    redact_urls(&mut output);
    redact_bearer(&mut output);
    for key in SECRET_KEYS.iter().chain(BODY_KEYS) {
        redact_assignment(&mut output, key, BODY_KEYS.contains(key));
    }
    let mut cursor = 0;
    while let Some(offset) = output[cursor..].to_ascii_lowercase().find("sk-") {
        let start = cursor + offset;
        let end = output[start..]
            .find(|c: char| !c.is_ascii_alphanumeric() && c != '-' && c != '_')
            .map_or(output.len(), |n| start + n);
        if end - start >= 11 {
            output.replace_range(start..end, "[REDACTED]");
            cursor = start + 10;
        } else {
            cursor = end;
        }
    }
    for prefix in ["\\users\\", "/users/", "/home/"] {
        let mut cursor = 0;
        while let Some(offset) = output[cursor..].to_ascii_lowercase().find(prefix) {
            let start = cursor + offset + prefix.len();
            let end = output[start..]
                .find(|c: char| c == '/' || c == '\\' || c.is_whitespace() || c == '"' || c == '\'')
                .map_or(output.len(), |n| start + n);
            output.replace_range(start..end, "[REDACTED]");
            cursor = start + 10;
        }
    }
    output
}

fn redact_urls(output: &mut String) {
    for scheme in ["postgres://", "postgresql://", "http://", "https://"] {
        let mut cursor = 0;
        while let Some(offset) = output[cursor..].to_ascii_lowercase().find(scheme) {
            let start = cursor + offset;
            let end = output[start..]
                .find(|c: char| {
                    c.is_whitespace() || matches!(c, '"' | '\'' | '<' | '>' | ')' | ',' | ';')
                })
                .map_or(output.len(), |n| start + n);
            let token = &output[start..end];
            let authority_end = token[scheme.len()..]
                .find(['/', '?', '#'])
                .map_or(token.len(), |n| scheme.len() + n);
            let mut safe = if let Some(at) = token[..authority_end].rfind('@') {
                format!("{}[REDACTED]@{}", &token[..scheme.len()], &token[at + 1..])
            } else {
                token.to_string()
            };
            if let Some(query) = safe.find(['?', '#']) {
                safe.truncate(query);
                safe.push_str("?[REDACTED]");
            }
            output.replace_range(start..end, &safe);
            cursor = start + safe.len();
        }
    }
}

fn redact_bearer(output: &mut String) {
    let mut cursor = 0;
    while let Some(offset) = output[cursor..].to_ascii_lowercase().find("bearer") {
        let end_word = cursor + offset + 6;
        if !output[end_word..].starts_with(char::is_whitespace) {
            cursor = end_word;
            continue;
        }
        let start = end_word + output[end_word..].len() - output[end_word..].trim_start().len();
        let end = output[start..]
            .find(|c: char| {
                c.is_whitespace() || matches!(c, ',' | ';' | '"' | '\'' | '<' | '>' | '}')
            })
            .map_or(output.len(), |n| start + n);
        output.replace_range(start..end, "[REDACTED]");
        cursor = start + 10;
    }
}

fn redact_assignment(output: &mut String, key: &str, body: bool) {
    let mut cursor = 0;
    while let Some(offset) = output[cursor..].to_ascii_lowercase().find(key) {
        let start_key = cursor + offset;
        let mut index = start_key + key.len();
        cursor = index;
        if start_key > 0 && output.as_bytes()[start_key - 1].is_ascii_alphanumeric() {
            continue;
        }
        if output[index..].starts_with(['"', '\'']) {
            index += 1;
        }
        while output
            .as_bytes()
            .get(index)
            .is_some_and(u8::is_ascii_whitespace)
        {
            index += 1;
        }
        if !output[index..].starts_with([':', '=']) {
            continue;
        }
        index += 1;
        while output
            .as_bytes()
            .get(index)
            .is_some_and(u8::is_ascii_whitespace)
        {
            index += 1;
        }
        let start = index;
        if key.ends_with("authorization") {
            if let Some(space) = output[start..].find(char::is_whitespace) {
                let scheme = &output[start..start + space];
                if scheme.eq_ignore_ascii_case("basic") || scheme.eq_ignore_ascii_case("bearer") {
                    let whitespace = &output[start + space..];
                    let token_start =
                        start + space + whitespace.len() - whitespace.trim_start().len();
                    let end = output[token_start..]
                        .find(|c: char| {
                            c.is_whitespace() || matches!(c, ',' | ';' | '"' | '\'' | '}')
                        })
                        .map_or(output.len(), |n| token_start + n);
                    output.replace_range(start..end, "[REDACTED]");
                    cursor = start + 10;
                    continue;
                }
            }
        }
        if output[start..].starts_with("[REDACTED]") {
            cursor = start + 10;
            continue;
        }
        if let Some(quote) = output
            .as_bytes()
            .get(index)
            .copied()
            .filter(|c| *c == b'"' || *c == b'\'')
        {
            index += 1;
            let value_start = index;
            let mut escaped = false;
            while index < output.len() {
                let ch = output.as_bytes()[index];
                if !escaped && ch == quote {
                    break;
                }
                escaped = !escaped && ch == b'\\';
                index += 1;
            }
            output.replace_range(value_start..index, "[REDACTED]");
            cursor = value_start + 10;
        } else {
            let end = output[start..]
                .find(|c: char| {
                    if body {
                        c == '\n' || c == '\r'
                    } else {
                        c.is_whitespace() || matches!(c, ',' | ';' | '}' | ']' | '"' | '\'')
                    }
                })
                .map_or(output.len(), |n| start + n);
            output.replace_range(start..end, "[REDACTED]");
            cursor = start + 10;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn redacts_labeled_secrets_bodies_urls_and_user_paths() {
        let input = r#"中 Authorization: bEaReR abc-private api_key="xyz-private" password='space private' https://user:pw@example.invalid/p?session=private#secret C:\Users\Owner\file prompt="用户正文" sk-proj-testabcdef"#;
        let safe = redact_text(input);
        for secret in [
            "abc-private",
            "xyz-private",
            "space private",
            "user:pw",
            "session=private",
            "#secret",
            "Owner",
            "用户正文",
            "testabcdef",
        ] {
            assert!(!safe.contains(secret), "{safe}");
        }
        assert!(safe.contains("example.invalid/p"));
        assert_eq!(redact_text(&safe), safe);
    }
    #[test]
    fn basic_and_quoted_credentials_are_fully_hidden() {
        let value="Authorization: Basic QA-BASIC-PRIVATE\nProxy-Authorization: bAsIc\tQA-BASIC-TAB\npassword='space private'\nCookie: session=QA-COOKIE; other=QA-COOKIE-2";
        let safe = redact_text(value);
        for secret in ["QA-BASIC", "space private", "QA-COOKIE"] {
            assert!(!safe.contains(secret), "{safe}");
        }
        assert_eq!(redact_text(&safe), safe);
    }
    #[test]
    fn sensitive_key_keeps_usage_counts() {
        assert!(sensitive_key("api_key"));
        assert!(sensitive_key("prompt"));
        assert!(!sensitive_key("total_tokens"));
        assert!(!sensitive_key("technical_message"));
    }
}
