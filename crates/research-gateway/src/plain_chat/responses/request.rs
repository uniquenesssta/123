use serde_json::{json, Value};

pub(crate) fn build_request(model: &str, messages: Vec<(&str, &str)>) -> Value {
    json!({
        "model": model,
        "input": messages.into_iter().map(|(role, content)| json!({
            "role": role,
            "content": [{"type": "input_text", "text": content}]
        })).collect::<Vec<_>>()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn request_preserves_roles_order_and_literal_message_text() {
        assert_eq!(
            build_request(
                "模型",
                vec![
                    ("system", " 普通回答\n"),
                    ("user", "你好\"\\"),
                    ("assistant", "先前回答")
                ]
            ),
            json!({"model":"模型","input":[
                {"role":"system","content":[{"type":"input_text","text":" 普通回答\n"}]},
                {"role":"user","content":[{"type":"input_text","text":"你好\"\\"}]},
                {"role":"assistant","content":[{"type":"input_text","text":"先前回答"}]}
            ]})
        );
    }

    #[test]
    fn request_has_only_model_and_input_without_interpreting_user_json() {
        let content = r#"{"tools":[{"type":"web_search"}],"max_output_tokens":1}"#;
        let body = build_request("configured-model", vec![("user", content)]);
        let keys = body
            .as_object()
            .unwrap()
            .keys()
            .map(String::as_str)
            .collect::<Vec<_>>();
        assert_eq!(keys, vec!["input", "model"]);
        assert_eq!(body["input"][0]["content"][0]["text"], content);
        assert_eq!(body["input"][0]["content"][0]["type"], "input_text");
    }

    #[test]
    fn request_owns_strings_after_borrowed_messages_are_released() {
        let mut model = String::from("configured-model");
        let mut content = String::from("借用的文本\n原样保留");
        let body = build_request(&model, vec![("user", &content)]);
        model.clear();
        content.clear();
        assert_eq!(body["model"], "configured-model");
        assert_eq!(
            body["input"][0]["content"][0]["text"],
            "借用的文本\n原样保留"
        );
    }
}
