use super::{invalid_state::invalid_state, optional::optional_trimmed_text};
use crate::PersistenceResult;
use serde_json::Value;
use uuid::Uuid;

pub(crate) fn required_uuid(raw: &str, key: &str) -> PersistenceResult<Uuid> {
    Uuid::parse_str(raw).map_err(|error| invalid_state(format!("{key} 不是有效 UUID：{error}")))
}

pub(crate) fn optional_uuid(value: &Value, key: &str) -> PersistenceResult<Option<Uuid>> {
    let Some(raw) = optional_trimmed_text(value.get(key), key, "UUID 字符串")? else {
        return Ok(None);
    };
    required_uuid(raw, key).map(Some)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn optional_uuid_accepts_missing_null_and_blank_values() {
        assert_eq!(optional_uuid(&json!({}), "id").unwrap(), None);
        assert_eq!(optional_uuid(&json!({"id": null}), "id").unwrap(), None);
        assert_eq!(optional_uuid(&json!({"id": "  "}), "id").unwrap(), None);
    }

    #[test]
    fn optional_uuid_trims_present_values_without_relaxing_required_uuid() {
        let id = Uuid::new_v4();
        assert_eq!(
            optional_uuid(&json!({"id": format!("  {id}  ")}), "id").unwrap(),
            Some(id)
        );
        assert!(required_uuid(&format!(" {id} "), "snapshot.snapshot_id").is_err());
    }
}
