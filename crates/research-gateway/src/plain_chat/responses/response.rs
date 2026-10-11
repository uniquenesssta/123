use serde_json::Value;

pub(crate) fn extract_text(value: &Value) -> Option<String> {
    if let Some(text) = value.get("output_text").and_then(Value::as_str) {
        if !text.trim().is_empty() {
            return Some(text.to_string());
        }
    }
    let mut parts = Vec::new();
    for item in value
        .get("output")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
    {
        if let Some(text) = item.as_str() {
            parts.push(text.to_string());
            continue;
        }
        for content in item
            .get("content")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
        {
            if let Some(text) = content.as_str() {
                parts.push(text.to_string());
            } else if let Some(text) = content
                .get("text")
                .and_then(Value::as_str)
                .or_else(|| content.get("content").and_then(Value::as_str))
            {
                parts.push(text.to_string());
            }
        }
    }
    (!parts.is_empty()).then(|| parts.join("\n"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn top_level_text_has_priority_without_trimming_the_raw_text() {
        assert_eq!(
            extract_text(&json!({
                "output_text":" 顶层\n ", "output":["嵌套"],
                "choices":[{"message":{"content":"其他协议"}}]
            })),
            Some(" 顶层\n ".to_string())
        );
    }

    #[test]
    fn nested_text_keeps_order_alias_priority_and_tolerates_unknown_shapes() {
        let value = json!({"output":[
            " 第一项 ", null, 4, {"content":"非数组"},
            {"type":"unknown","content":["字符串", null, 8,
                {"text":"text优先","content":"不选"},
                {"text":7,"content":"content别名"}, {"other":"忽略"}]},
            {"content":[{"text":"最后\n一项"}]}
        ]});
        assert_eq!(
            extract_text(&value),
            Some(" 第一项 \n字符串\ntext优先\ncontent别名\n最后\n一项".to_string())
        );
    }

    #[test]
    fn blank_top_level_falls_back_but_nested_blank_remains_some() {
        for top in [json!(" \n"), json!(4), Value::Null] {
            assert_eq!(
                extract_text(&json!({"output_text":top,"output":["嵌套"]})),
                Some("嵌套".to_string())
            );
        }
        for blank in ["", " \n"] {
            assert_eq!(
                extract_text(&json!({"output":[{"content":[{"text":blank}]}]})),
                Some(blank.to_string())
            );
        }
    }

    #[test]
    fn absent_or_unrecognized_parts_return_none_without_chat_fallback() {
        for value in [
            Value::Null,
            json!({}),
            json!({"output":"文本"}),
            json!({"output":[null,{"content":[{"refusal":"拒绝"}]}]}),
            json!({"choices":[{"message":{"content":"chat"}}]}),
        ] {
            assert_eq!(extract_text(&value), None);
        }
    }

    #[test]
    fn unicode_fences_and_json_are_returned_as_literal_text() {
        let text = "```json\n{\"回答\":\"足球⚽\",\"tools\":[]}\n```";
        assert_eq!(
            extract_text(&json!({"output":[{"content":[{"text":text}]}]})),
            Some(text.to_string())
        );
    }
}
