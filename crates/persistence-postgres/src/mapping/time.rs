use super::invalid_state::invalid_state;
use crate::PersistenceResult;
use chrono::{DateTime, Utc};
use serde_json::Value;

pub(crate) fn required_datetime(
    value: Option<&Value>,
    key: &str,
) -> PersistenceResult<DateTime<Utc>> {
    let raw = value
        .and_then(Value::as_str)
        .ok_or_else(|| invalid_state(format!("{key} 必须是 RFC3339 时间")))?;
    DateTime::parse_from_rfc3339(raw)
        .map(|value| value.with_timezone(&Utc))
        .map_err(|error| invalid_state(format!("{key} 时间无效：{error}")))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn parses_rfc3339_into_utc() {
        let value = json!("2026-08-13T11:53:00+08:00");
        let parsed = required_datetime(Some(&value), "snapshot.frozen_at").unwrap();
        assert_eq!(parsed.to_rfc3339(), "2026-08-13T03:53:00+00:00");
    }

    #[test]
    fn rejects_non_string_time_with_existing_error_semantics() {
        let value = json!(123);
        let error = required_datetime(Some(&value), "snapshot.frozen_at").unwrap_err();
        assert!(
            matches!(error, crate::PersistenceError::InvalidState(message) if message == "snapshot.frozen_at 必须是 RFC3339 时间")
        );
    }
}
