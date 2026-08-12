use crate::{ApplicationError, ApplicationResult};
use chrono::{DateTime, NaiveDate, Utc};
use football_domain::AvailabilityStatus;
use serde_json::Value;
use uuid::Uuid;

pub(super) fn required_text(value: &Value, key: &str) -> ApplicationResult<String> {
    value
        .get(key)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToString::to_string)
        .ok_or_else(|| ApplicationError::Validation(format!("数据库提案缺少字段：{key}")))
}

pub(super) fn optional_text(value: &Value, key: &str) -> Option<String> {
    value
        .get(key)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToString::to_string)
}

pub(super) fn required_uuid(value: &Value, key: &str) -> ApplicationResult<Uuid> {
    let raw = required_text(value, key)?;
    Uuid::parse_str(&raw)
        .map_err(|error| ApplicationError::Validation(format!("字段{key}不是有效UUID：{error}")))
}

pub(super) fn optional_uuid(value: &Value, key: &str) -> ApplicationResult<Option<Uuid>> {
    optional_text(value, key)
        .map(|raw| {
            Uuid::parse_str(&raw).map_err(|error| {
                ApplicationError::Validation(format!("字段{key}不是有效UUID：{error}"))
            })
        })
        .transpose()
}

pub(super) fn required_f64(value: &Value, key: &str) -> ApplicationResult<f64> {
    optional_f64(value, key)
        .ok_or_else(|| ApplicationError::Validation(format!("数据库提案缺少数值字段：{key}")))
}

pub(super) fn optional_f64(value: &Value, key: &str) -> Option<f64> {
    value.get(key).and_then(Value::as_f64)
}

pub(super) fn optional_i32(value: &Value, key: &str) -> Option<i32> {
    value
        .get(key)
        .and_then(Value::as_i64)
        .and_then(|value| i32::try_from(value).ok())
}

pub(super) fn optional_i16(value: &Value, key: &str) -> Option<i16> {
    value
        .get(key)
        .and_then(Value::as_i64)
        .and_then(|value| i16::try_from(value).ok())
}

pub(super) fn required_datetime(value: &Value, key: &str) -> ApplicationResult<DateTime<Utc>> {
    let raw = required_text(value, key)?;
    DateTime::parse_from_rfc3339(&raw)
        .map(|value| value.with_timezone(&Utc))
        .map_err(|error| ApplicationError::Validation(format!("字段{key}时间无效：{error}")))
}

pub(super) fn optional_datetime(
    value: &Value,
    key: &str,
) -> ApplicationResult<Option<DateTime<Utc>>> {
    optional_text(value, key)
        .map(|raw| {
            DateTime::parse_from_rfc3339(&raw)
                .map(|value| value.with_timezone(&Utc))
                .map_err(|error| {
                    ApplicationError::Validation(format!("字段{key}时间无效：{error}"))
                })
        })
        .transpose()
}

pub(super) fn optional_date(value: &Value, key: &str) -> ApplicationResult<Option<NaiveDate>> {
    optional_text(value, key)
        .map(|raw| {
            NaiveDate::parse_from_str(&raw, "%Y-%m-%d").map_err(|error| {
                ApplicationError::Validation(format!("字段{key}日期无效：{error}"))
            })
        })
        .transpose()
}

pub(super) fn availability_status(value: String) -> ApplicationResult<AvailabilityStatus> {
    match value.as_str() {
        "available" => Ok(AvailabilityStatus::Available),
        "doubtful" => Ok(AvailabilityStatus::Doubtful),
        "unavailable" => Ok(AvailabilityStatus::Unavailable),
        "injured" => Ok(AvailabilityStatus::Injured),
        "suspended" => Ok(AvailabilityStatus::Suspended),
        "rested" => Ok(AvailabilityStatus::Rested),
        "returning" => Ok(AvailabilityStatus::Returning),
        "unknown" => Ok(AvailabilityStatus::Unknown),
        _ => Err(ApplicationError::Validation(format!(
            "未知球员可用性状态：{value}"
        ))),
    }
}
