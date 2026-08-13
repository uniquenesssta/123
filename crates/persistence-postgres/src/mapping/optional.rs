use super::invalid_state::invalid_state;
use crate::PersistenceResult;
use serde_json::Value;

pub(super) fn optional_trimmed_text<'a>(
    value: Option<&'a Value>,
    key: &str,
    expected_type: &str,
) -> PersistenceResult<Option<&'a str>> {
    let Some(value) = value else {
        return Ok(None);
    };
    if value.is_null() {
        return Ok(None);
    }
    let text = value
        .as_str()
        .ok_or_else(|| invalid_state(format!("{key} 必须是 {expected_type}")))?
        .trim();
    if text.is_empty() {
        Ok(None)
    } else {
        Ok(Some(text))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn normalizes_missing_null_blank_and_present_text() {
        assert_eq!(optional_trimmed_text(None, "id", "字符串").unwrap(), None);
        assert_eq!(
            optional_trimmed_text(Some(&Value::Null), "id", "字符串").unwrap(),
            None
        );
        let blank = json!("   ");
        assert_eq!(
            optional_trimmed_text(Some(&blank), "id", "字符串").unwrap(),
            None
        );
        let text = json!("  value  ");
        assert_eq!(
            optional_trimmed_text(Some(&text), "id", "字符串").unwrap(),
            Some("value")
        );
    }
}
