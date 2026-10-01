use crate::{PersistenceError, PersistenceResult};
use chrono::{DateTime, NaiveDate, Utc};
use football_domain::AvailabilityStatus;
use serde_json::{Map, Value};
use std::collections::HashMap;
use uuid::Uuid;

pub(super) fn resolve_import_match(
    values: &Map<String, Value>,
    map: &HashMap<String, Uuid>,
) -> PersistenceResult<Uuid> {
    if let Some(id) = payload_optional_uuid(values, "_resolved_match_id")? {
        return Ok(id);
    }
    let key = values
        .get("_deferred_match_key")
        .and_then(Value::as_str)
        .unwrap_or_else(|| {
            values
                .get("match_key")
                .and_then(Value::as_str)
                .unwrap_or("")
        });
    map.get(key)
        .copied()
        .ok_or_else(|| PersistenceError::InvalidState(format!("无法解析比赛：{key}")))
}

pub(super) fn required(values: &Map<String, Value>, key: &str) -> PersistenceResult<String> {
    let value = text(values, key);
    if value.is_empty() {
        Err(PersistenceError::InvalidState(format!(
            "缺少必填字段：{key}"
        )))
    } else {
        Ok(value)
    }
}

pub(super) fn text(values: &Map<String, Value>, key: &str) -> String {
    values
        .get(key)
        .and_then(Value::as_str)
        .unwrap_or("")
        .trim()
        .to_string()
}

pub(super) fn optional_text(values: &Map<String, Value>, key: &str) -> Option<String> {
    let value = text(values, key);
    if value.is_empty() {
        None
    } else {
        Some(value)
    }
}

pub(super) fn default_text(values: &Map<String, Value>, key: &str, default: &str) -> String {
    optional_text(values, key).unwrap_or_else(|| default.to_string())
}

pub(super) fn optional_uuid(
    values: &Map<String, Value>,
    key: &str,
) -> PersistenceResult<Option<Uuid>> {
    let value = text(values, key);
    if value.is_empty() {
        Ok(None)
    } else {
        Uuid::parse_str(&value)
            .map(Some)
            .map_err(|error| PersistenceError::InvalidState(format!("{key} UUID 无效：{error}")))
    }
}

pub(super) fn payload_optional_uuid(
    values: &Map<String, Value>,
    key: &str,
) -> PersistenceResult<Option<Uuid>> {
    match values.get(key) {
        None | Some(Value::Null) => Ok(None),
        Some(Value::String(value)) if value.is_empty() => Ok(None),
        Some(value) => serde_json::from_value(value.clone())
            .map(Some)
            .map_err(PersistenceError::Serialization),
    }
}

pub(super) fn payload_uuid(values: &Map<String, Value>, key: &str) -> PersistenceResult<Uuid> {
    payload_optional_uuid(values, key)?
        .ok_or_else(|| PersistenceError::InvalidState(format!("缺少内部字段：{key}")))
}

pub(super) fn optional_date(
    values: &Map<String, Value>,
    key: &str,
) -> PersistenceResult<Option<NaiveDate>> {
    let value = text(values, key);
    if value.is_empty() {
        Ok(None)
    } else {
        NaiveDate::parse_from_str(&value, "%Y-%m-%d")
            .map(Some)
            .map_err(|error| PersistenceError::InvalidState(format!("{key} 日期无效：{error}")))
    }
}

pub(super) fn required_datetime(
    values: &Map<String, Value>,
    key: &str,
) -> PersistenceResult<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(&required(values, key)?)
        .map(|value| value.with_timezone(&Utc))
        .map_err(|error| PersistenceError::InvalidState(format!("{key} 时间无效：{error}")))
}

pub(super) fn optional_f64(
    values: &Map<String, Value>,
    key: &str,
) -> PersistenceResult<Option<f64>> {
    let value = text(values, key);
    if value.is_empty() {
        Ok(None)
    } else {
        value
            .parse()
            .map(Some)
            .map_err(|error| PersistenceError::InvalidState(format!("{key} 数值无效：{error}")))
    }
}

pub(super) fn required_f64(values: &Map<String, Value>, key: &str) -> PersistenceResult<f64> {
    optional_f64(values, key)?
        .ok_or_else(|| PersistenceError::InvalidState(format!("缺少数值：{key}")))
}

pub(super) fn optional_i16(
    values: &Map<String, Value>,
    key: &str,
) -> PersistenceResult<Option<i16>> {
    let value = text(values, key);
    if value.is_empty() {
        Ok(None)
    } else {
        value
            .parse()
            .map(Some)
            .map_err(|error| PersistenceError::InvalidState(format!("{key} 整数无效：{error}")))
    }
}

pub(super) fn optional_i32(
    values: &Map<String, Value>,
    key: &str,
) -> PersistenceResult<Option<i32>> {
    let value = text(values, key);
    if value.is_empty() {
        Ok(None)
    } else {
        value
            .parse()
            .map(Some)
            .map_err(|error| PersistenceError::InvalidState(format!("{key} 整数无效：{error}")))
    }
}

pub(super) fn optional_bool(
    values: &Map<String, Value>,
    key: &str,
) -> PersistenceResult<Option<bool>> {
    let value = text(values, key).to_lowercase();
    match value.as_str() {
        "" => Ok(None),
        "true" | "1" | "yes" => Ok(Some(true)),
        "false" | "0" | "no" => Ok(Some(false)),
        _ => Err(PersistenceError::InvalidState(format!("{key} 布尔值无效"))),
    }
}

pub(super) fn required_bool(values: &Map<String, Value>, key: &str) -> PersistenceResult<bool> {
    optional_bool(values, key)?
        .ok_or_else(|| PersistenceError::InvalidState(format!("缺少布尔字段：{key}")))
}

pub(super) fn parse_source_urls(values: &Map<String, Value>, key: &str) -> Vec<String> {
    let raw = text(values, key);
    let mut urls = raw
        .split([';', '\n', '\r', ','])
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .collect::<Vec<_>>();
    urls.sort();
    urls.dedup();
    urls
}

pub(super) fn normalize(value: &str) -> String {
    value
        .to_lowercase()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

pub(super) fn availability_from_str(value: &str) -> PersistenceResult<AvailabilityStatus> {
    match value {
        "available" => Ok(AvailabilityStatus::Available),
        "doubtful" => Ok(AvailabilityStatus::Doubtful),
        "unavailable" => Ok(AvailabilityStatus::Unavailable),
        "injured" => Ok(AvailabilityStatus::Injured),
        "suspended" => Ok(AvailabilityStatus::Suspended),
        "rested" => Ok(AvailabilityStatus::Rested),
        "returning" => Ok(AvailabilityStatus::Returning),
        "unknown" => Ok(AvailabilityStatus::Unknown),
        _ => Err(PersistenceError::InvalidState("未知可用状态".to_string())),
    }
}
