use crate::{PersistenceError, PersistenceResult};
use chrono::{DateTime, Duration, NaiveDate, NaiveDateTime, SecondsFormat, Utc};
use serde_json::{json, Map, Value};
use std::collections::HashSet;
use uuid::Uuid;

pub(super) fn object(value: &Value) -> PersistenceResult<&Map<String, Value>> {
    value
        .as_object()
        .ok_or_else(|| PersistenceError::InvalidState("Excel 行内容不是对象".into()))
}

pub(super) fn object_mut(value: &mut Value) -> PersistenceResult<&mut Map<String, Value>> {
    value
        .as_object_mut()
        .ok_or_else(|| PersistenceError::InvalidState("Excel 行内容不是对象".into()))
}

pub(super) fn text(values: &Map<String, Value>, key: &str) -> Option<String> {
    values
        .get(key)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .map(str::to_string)
}

pub(super) fn normalize_team_type(value: Option<String>) -> PersistenceResult<Option<String>> {
    let Some(raw) = value else {
        return Ok(None);
    };
    let key = raw
        .trim()
        .to_lowercase()
        .chars()
        .filter(|character| {
            !character.is_whitespace() && !matches!(character, '_' | '-' | '\'' | '’' | '.' | '/')
        })
        .collect::<String>();
    let canonical = match key.as_str() {
        "club" | "clubs" | "clubteam" | "俱乐部" | "俱乐部队" => "club",
        "national" | "nationalteam" | "seniornational" | "seniornationalteam" | "国家队"
        | "国家代表队" | "成年国家队" => "national",
        "reserve" | "reserves" | "reserveteam" | "bteam" | "secondteam" | "预备队" | "二队"
        | "b队" => "reserve",
        "youth" | "youthteam" | "academy" | "academyteam" | "u18" | "u19" | "u20" | "u21"
        | "u23" | "青年队" | "青训队" | "梯队" => "youth",
        "women" | "womens" | "womenteam" | "womensteam" | "female" | "女足" | "女子队"
        | "女子足球队" => "women",
        "other" | "others" | "其他" | "其它" => "other",
        _ => {
            return Err(PersistenceError::InvalidState(format!(
                "球队类型 team_type={raw} 无效；允许 club/national/reserve/youth/women/other，常见别名 national_team、国家队、俱乐部、预备队、青年队、女足也可自动识别"
            )));
        }
    };
    Ok(Some(canonical.to_string()))
}

pub(super) fn normalize_team_type_payload(payload: &mut Value) -> PersistenceResult<bool> {
    let values = object_mut(payload)?;
    let current = text(values, "team_type");
    let Some(canonical) = normalize_team_type(current.clone())? else {
        return Ok(false);
    };
    if current.as_deref() == Some(canonical.as_str()) {
        return Ok(false);
    }
    values.insert("team_type".into(), Value::String(canonical));
    Ok(true)
}

pub(super) fn require_text(values: &Map<String, Value>, key: &str) -> PersistenceResult<String> {
    text(values, key).ok_or_else(|| PersistenceError::InvalidState(format!("缺少必填字段 {key}")))
}

pub(super) fn optional_uuid(
    values: &Map<String, Value>,
    key: &str,
) -> PersistenceResult<Option<Uuid>> {
    match text(values, key) {
        Some(v) => Uuid::parse_str(&v)
            .map(Some)
            .map_err(|_| PersistenceError::InvalidState(format!("{key} 不是有效 UUID"))),
        None => Ok(None),
    }
}

pub(super) fn resolved_uuid(values: &Map<String, Value>, prefix: &str) -> PersistenceResult<Uuid> {
    optional_uuid(values, &format!("_resolved_{prefix}_id"))?
        .or(optional_uuid(values, &format!("{prefix}_id"))?)
        .ok_or_else(|| PersistenceError::InvalidState(format!("缺少已解析的 {prefix}_id")))
}

pub(super) fn parse_date(value: String, key: &str) -> PersistenceResult<NaiveDate> {
    NaiveDate::parse_from_str(&value, "%Y-%m-%d")
        .map_err(|_| PersistenceError::InvalidState(format!("{key} 必须是 YYYY-MM-DD")))
}

pub(super) fn optional_date(
    values: &Map<String, Value>,
    key: &str,
) -> PersistenceResult<Option<NaiveDate>> {
    text(values, key).map(|v| parse_date(v, key)).transpose()
}

fn parse_datetime(value: String, key: &str) -> PersistenceResult<DateTime<Utc>> {
    let value = value.trim();
    if let Ok(parsed) = DateTime::parse_from_rfc3339(value) {
        return Ok(parsed.with_timezone(&Utc));
    }
    for format in [
        "%Y-%m-%dT%H:%M:%S",
        "%Y-%m-%dT%H:%M:%S%.f",
        "%Y-%m-%d %H:%M:%S",
        "%Y-%m-%d %H:%M:%S%.f",
        "%Y/%m/%d %H:%M:%S",
        "%Y/%m/%d %H:%M:%S%.f",
    ] {
        if let Ok(parsed) = NaiveDateTime::parse_from_str(value, format) {
            return Ok(DateTime::from_naive_utc_and_offset(parsed, Utc));
        }
    }
    for format in ["%Y-%m-%d", "%Y/%m/%d", "%Y年%m月%d日"] {
        if let Ok(parsed) = NaiveDate::parse_from_str(value, format) {
            let midnight = parsed.and_hms_opt(0, 0, 0).ok_or_else(|| {
                PersistenceError::InvalidState(format!("{key} 日期无法转换为时间"))
            })?;
            return Ok(DateTime::from_naive_utc_and_offset(midnight, Utc));
        }
    }
    if let Ok(serial) = value.parse::<f64>() {
        if serial.is_finite() && (1.0..=1_000_000.0).contains(&serial) {
            let whole_days = serial.floor() as i64;
            let seconds = ((serial - whole_days as f64) * 86_400.0).round() as i64;
            let epoch = NaiveDate::from_ymd_opt(1899, 12, 30)
                .and_then(|date| date.and_hms_opt(0, 0, 0))
                .ok_or_else(|| PersistenceError::InvalidState("Excel 时间基准无效".into()))?;
            if let Some(parsed) = epoch
                .checked_add_signed(Duration::days(whole_days))
                .and_then(|date_time| date_time.checked_add_signed(Duration::seconds(seconds)))
            {
                return Ok(DateTime::from_naive_utc_and_offset(parsed, Utc));
            }
        }
    }
    Err(PersistenceError::InvalidState(format!(
        "{key} 必须是有效时间；支持 ISO 8601、YYYY-MM-DD、YYYY-MM-DD HH:MM:SS 或 Excel 日期单元格"
    )))
}

fn canonical_datetime(value: String, key: &str) -> PersistenceResult<String> {
    parse_datetime(value, key)
        .map(|date_time| date_time.to_rfc3339_opts(SecondsFormat::AutoSi, true))
}

pub(super) fn normalize_monthly_datetime_payload(payload: &mut Value) -> PersistenceResult<bool> {
    let values = object_mut(payload)?;
    let mut changed = false;
    for key in ["verified_at", "observed_at"] {
        let Some(current) = text(values, key) else {
            continue;
        };
        let canonical = canonical_datetime(current.clone(), key)?;
        if current != canonical {
            values.insert(key.into(), Value::String(canonical));
            changed = true;
        }
    }
    Ok(changed)
}

fn canonical_date(value: String, key: &str) -> PersistenceResult<String> {
    parse_datetime(value, key).map(|date_time| date_time.date_naive().to_string())
}

pub(super) fn normalize_point_observation_window_payload(
    payload: &mut Value,
) -> PersistenceResult<bool> {
    let values = object_mut(payload)?;
    let current_start = text(values, "window_start");
    let current_end = text(values, "window_end");
    let anchor_date = text(values, "observed_at")
        .or_else(|| text(values, "verified_at"))
        .map(|value| canonical_date(value, "observed_at"))
        .transpose()?;

    let start = current_start
        .clone()
        .map(|value| canonical_date(value, "window_start"))
        .transpose()?;
    let end = current_end
        .clone()
        .map(|value| canonical_date(value, "window_end"))
        .transpose()?;

    let (normalized_start, normalized_end) = match (start, end) {
        (Some(start), Some(end)) => (start, end),
        (Some(start), None) => (start.clone(), start),
        (None, Some(end)) => (end.clone(), end),
        (None, None) => {
            let date = anchor_date.ok_or_else(|| {
                PersistenceError::InvalidState(
                    "球队能力或战术观察缺少 window_start/window_end，且没有 observed_at/verified_at 可用于生成点时窗口".into(),
                )
            })?;
            (date.clone(), date)
        }
    };

    let start_date = parse_date(normalized_start.clone(), "window_start")?;
    let end_date = parse_date(normalized_end.clone(), "window_end")?;
    if end_date < start_date {
        return Err(PersistenceError::InvalidState(
            "window_end 不能早于 window_start".into(),
        ));
    }

    let mut changed = false;
    if current_start.as_deref() != Some(normalized_start.as_str()) {
        values.insert("window_start".into(), Value::String(normalized_start));
        changed = true;
    }
    if current_end.as_deref() != Some(normalized_end.as_str()) {
        values.insert("window_end".into(), Value::String(normalized_end));
        changed = true;
    }
    Ok(changed)
}

pub(super) fn optional_datetime(
    values: &Map<String, Value>,
    key: &str,
) -> PersistenceResult<Option<DateTime<Utc>>> {
    text(values, key)
        .map(|value| parse_datetime(value, key))
        .transpose()
}

pub(super) fn parse_i32(value: String, key: &str) -> PersistenceResult<i32> {
    value
        .parse()
        .map_err(|_| PersistenceError::InvalidState(format!("{key} 必须是整数")))
}

pub(super) fn parse_i32_default(
    value: Option<&str>,
    default: i32,
    key: &str,
) -> PersistenceResult<i32> {
    value
        .map(|v| parse_i32(v.to_string(), key))
        .transpose()
        .map(|v| v.unwrap_or(default))
}

pub(super) fn optional_i16(
    values: &Map<String, Value>,
    key: &str,
) -> PersistenceResult<Option<i16>> {
    text(values, key)
        .map(|v| {
            v.parse()
                .map_err(|_| PersistenceError::InvalidState(format!("{key} 必须是整数")))
        })
        .transpose()
}

pub(super) fn optional_f64(
    values: &Map<String, Value>,
    key: &str,
) -> PersistenceResult<Option<f64>> {
    text(values, key)
        .map(|v| {
            v.parse()
                .map_err(|_| PersistenceError::InvalidState(format!("{key} 必须是数字")))
        })
        .transpose()
}

pub(super) fn parse_f64_default(
    value: Option<&str>,
    default: f64,
    key: &str,
) -> PersistenceResult<f64> {
    value
        .map(|v| {
            v.parse()
                .map_err(|_| PersistenceError::InvalidState(format!("{key} 必须是数字")))
        })
        .transpose()
        .map(|v| v.unwrap_or(default))
}

pub(super) fn optional_bool(
    values: &Map<String, Value>,
    key: &str,
) -> PersistenceResult<Option<bool>> {
    text(values, key).map(|v| parse_bool(&v, key)).transpose()
}

pub(super) fn parse_bool_default(value: Option<&str>, default: bool) -> PersistenceResult<bool> {
    value
        .map(|v| parse_bool(v, "boolean"))
        .transpose()
        .map(|v| v.unwrap_or(default))
}

fn parse_bool(value: &str, key: &str) -> PersistenceResult<bool> {
    match value.trim().to_ascii_lowercase().as_str() {
        "true" | "1" | "yes" | "是" => Ok(true),
        "false" | "0" | "no" | "否" => Ok(false),
        _ => Err(PersistenceError::InvalidState(format!(
            "{key} 必须是 true/false"
        ))),
    }
}

pub(super) fn source_urls(values: &Map<String, Value>) -> Vec<String> {
    text(values, "source_urls")
        .map(|v| {
            v.split(['\n', ';'])
                .map(str::trim)
                .filter(|v| !v.is_empty())
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default()
}

pub(super) fn clear_fields(values: &Map<String, Value>) -> HashSet<String> {
    text(values, "clear_fields")
        .map(|v| {
            v.split([',', '，', ';'])
                .map(str::trim)
                .filter(|v| !v.is_empty())
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default()
}

pub(super) fn common_metadata(values: &Map<String, Value>) -> Value {
    json!({
        "source_urls": source_urls(values),
        "verified_at": text(values, "verified_at"),
        "confidence": text(values, "confidence").and_then(|value| value.parse::<f64>().ok()),
        "formation_familiarity": text(values, "formation_familiarity")
            .and_then(|value| value.parse::<f64>().ok()),
        "notes": text(values, "notes"),
        "monthly_workbook": true
    })
}

pub(super) fn normalize_name(value: &str) -> String {
    value
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;
    use football_domain::TEAM_MONTHLY_FORMAT;
    #[test]
    fn source_urls_are_split_and_deduplicatable() {
        let values = Map::from_iter([(
            "source_urls".into(),
            Value::String("https://a.example\nhttps://b.example;https://c.example".into()),
        )]);
        assert_eq!(source_urls(&values).len(), 3);
    }
    #[test]
    fn blank_is_not_a_clear_instruction() {
        let values = Map::from_iter([("clear_fields".into(), Value::String(String::new()))]);
        assert!(clear_fields(&values).is_empty());
    }
    #[test]
    fn player_monthly_format_is_distinct_from_team_format() {
        assert_ne!(football_domain::PLAYER_MONTHLY_FORMAT, TEAM_MONTHLY_FORMAT);
    }
    #[test]
    fn team_type_aliases_are_normalized_before_database_write() {
        assert_eq!(
            normalize_team_type(Some("national_team".into())).expect("national alias"),
            Some("national".into())
        );
        assert_eq!(
            normalize_team_type(Some("国家队".into())).expect("Chinese national alias"),
            Some("national".into())
        );
        assert_eq!(
            normalize_team_type(Some("俱乐部".into())).expect("Chinese club alias"),
            Some("club".into())
        );
    }
    #[test]
    fn invalid_team_type_is_rejected_before_sql_constraint() {
        let error = normalize_team_type(Some("unsupported".into())).expect_err("invalid type");
        assert!(error.to_string().contains("球队类型"));
    }
    #[test]
    fn existing_batch_payload_is_canonicalized_before_commit() {
        let mut payload = json!({"team_type": "national_team"});
        assert!(normalize_team_type_payload(&mut payload).expect("normalize payload"));
        assert_eq!(payload["team_type"], "national");
        assert!(!normalize_team_type_payload(&mut payload).expect("idempotent payload"));
    }
    #[test]
    fn monthly_datetime_text_is_canonicalized_before_commit() {
        let mut payload = json!({
            "verified_at": "2026-07-01 00:00:00",
            "observed_at": "2026/07/02"
        });
        assert!(normalize_monthly_datetime_payload(&mut payload).expect("normalize datetime"));
        assert_eq!(payload["verified_at"], "2026-07-01T00:00:00Z");
        assert_eq!(payload["observed_at"], "2026-07-02T00:00:00Z");
        assert!(!normalize_monthly_datetime_payload(&mut payload).expect("idempotent datetime"));
    }
    #[test]
    fn monthly_datetime_accepts_excel_serial_cells() {
        assert_eq!(
            canonical_datetime("46204".into(), "verified_at").expect("Excel date"),
            "2026-07-01T00:00:00Z"
        );
        assert_eq!(
            canonical_datetime("46204.5".into(), "verified_at").expect("Excel datetime"),
            "2026-07-01T12:00:00Z"
        );
    }
    #[test]
    fn monthly_datetime_rejects_unrecognized_text_before_database_write() {
        let error =
            canonical_datetime("not-a-date".into(), "verified_at").expect_err("invalid datetime");
        assert!(error.to_string().contains("Excel 日期单元格"));
    }
    #[test]
    fn point_observation_window_uses_end_for_missing_start() {
        let mut payload = json!({
            "window_start": "",
            "window_end": "2026-07-18",
            "observed_at": "2026-07-18T00:00:00Z"
        });
        assert!(normalize_point_observation_window_payload(&mut payload)
            .expect("normalize point window"));
        assert_eq!(payload["window_start"], "2026-07-18");
        assert_eq!(payload["window_end"], "2026-07-18");
        assert!(!normalize_point_observation_window_payload(&mut payload)
            .expect("idempotent point window"));
    }
    #[test]
    fn point_observation_window_uses_observed_at_when_both_dates_are_blank() {
        let mut payload = json!({
            "observed_at": "2026-07-18T15:30:00Z"
        });
        assert!(
            normalize_point_observation_window_payload(&mut payload).expect("derive point window")
        );
        assert_eq!(payload["window_start"], "2026-07-18");
        assert_eq!(payload["window_end"], "2026-07-18");
    }
    #[test]
    fn point_observation_window_rejects_inverted_range() {
        let mut payload = json!({
            "window_start": "2026-07-19",
            "window_end": "2026-07-18",
            "observed_at": "2026-07-18T00:00:00Z"
        });
        let error = normalize_point_observation_window_payload(&mut payload)
            .expect_err("inverted point window");
        assert!(error
            .to_string()
            .contains("window_end 不能早于 window_start"));
    }
}
