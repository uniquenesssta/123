use serde_json::{json, Value};

pub(crate) fn build_request(model: &str, messages: Vec<(&str, &str)>) -> Value {
    json!({
        "model": model,
        "messages": messages.into_iter().map(|(role, content)| json!({
            "role": role,
            "content": content
        })).collect::<Vec<_>>()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chat_request_preserves_roles_order_and_string_content() {
        let body = build_request(
            "配置模型",
            vec![
                ("system", " 普通回答\n"),
                ("user", "足球⚽\"\\"),
                ("assistant", "先前回答"),
            ],
        );
        assert_eq!(
            body,
            json!({"model":"配置模型","messages":[
                {"role":"system","content":" 普通回答\n"},
                {"role":"user","content":"足球⚽\"\\"},
                {"role":"assistant","content":"先前回答"}
            ]})
        );
    }

    #[test]
    fn chat_request_has_only_model_and_messages_without_interpreting_json() {
        let text = r#"{"tools":[{"type":"web_search"}],"max_tokens":1}"#;
        let body = build_request("model", vec![("user", text)]);
        assert_eq!(body.as_object().unwrap().len(), 2);
        assert_eq!(body["model"], "model");
        assert_eq!(body["messages"][0]["content"], text);
        assert!(body.get("input").is_none());
    }

    #[test]
    fn chat_request_owns_strings_after_borrowed_messages_are_released() {
        let mut model = String::from("配置模型");
        let mut text = String::from(" 原样文本\n");
        let body = build_request(&model, vec![("user", &text)]);
        model.clear();
        text.clear();
        assert_eq!(body["model"], "配置模型");
        assert_eq!(body["messages"][0]["content"], " 原样文本\n");
    }
}
