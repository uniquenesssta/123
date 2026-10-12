use serde_json::Value;

pub(crate) fn extract_text(value: &Value) -> Option<String> {
    let choice = value.get("choices").and_then(Value::as_array)?.first()?;
    if let Some(text) = choice.get("text").and_then(Value::as_str) {
        if !text.trim().is_empty() {
            return Some(text.to_string());
        }
    }
    let content = choice.get("message")?.get("content")?;
    if let Some(text) = content.as_str() {
        return Some(text.to_string());
    }
    let mut parts = Vec::new();
    for item in content.as_array().into_iter().flatten() {
        if let Some(text) = item.as_str() {
            parts.push(text.to_string());
        } else if let Some(text) = item
            .get("text")
            .and_then(Value::as_str)
            .or_else(|| item.get("content").and_then(Value::as_str))
        {
            parts.push(text.to_string());
        }
    }
    (!parts.is_empty()).then(|| parts.join("\n"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn chat_legacy_text_has_priority_and_only_first_choice_is_read() {
        let value = json!({"choices":[
            {"text":" 旧式文本\n ","message":{"content":"message文本"}},
            {"text":"第二条不能覆盖"}
        ]});
        assert_eq!(extract_text(&value), Some(" 旧式文本\n ".to_string()));
        assert_eq!(
            extract_text(&json!({"choices":[{}, {"text":"第二条不能救回"}]})),
            None
        );
    }

    #[test]
    fn chat_blank_legacy_falls_back_but_message_blank_remains_some() {
        for legacy in [json!(" \n"), json!(4), Value::Null] {
            assert_eq!(
                extract_text(&json!({"choices":[{"text":legacy,"message":{"content":"消息"}}]})),
                Some("消息".to_string())
            );
        }
        for blank in ["", " \n"] {
            assert_eq!(
                extract_text(&json!({"choices":[{"message":{"content":blank}}]})),
                Some(blank.to_string())
            );
        }
    }

    #[test]
    fn chat_array_content_preserves_order_alias_priority_and_unknown_shapes() {
        let value = json!({"choices":[{"message":{"content":[
            " 第一项 ",null,5,{"text":"text优先","content":"不选"},
            {"text":4,"content":"content别名"},{"other":"忽略"},{"text":"最后\n一项"}
        ]}}]});
        assert_eq!(
            extract_text(&value),
            Some(" 第一项 \ntext优先\ncontent别名\n最后\n一项".to_string())
        );
    }

    #[test]
    fn chat_absent_or_unrecognized_first_choice_returns_none_without_responses_fallback() {
        for value in [
            Value::Null,
            json!({}),
            json!({"choices":null}),
            json!({"choices":[]}),
            json!({"choices":"文本"}),
            json!({"choices":[null]}),
            json!({"choices":[{"message":{"content":4}}]}),
            json!({"choices":[{"message":{"content":[null,{"refusal":"拒绝"}]}}]}),
            json!({"output_text":"Responses文本"}),
        ] {
            assert_eq!(extract_text(&value), None);
        }
    }

    #[test]
    fn chat_unicode_json_and_fences_stay_literal_regardless_of_finish_reason() {
        let text = "```json\n{\"回答\":\"足球⚽\"}\n```";
        assert_eq!(
            extract_text(
                &json!({"choices":[{"message":{"content":text},"finish_reason":"length"}]})
            ),
            Some(text.to_string())
        );
    }
}
