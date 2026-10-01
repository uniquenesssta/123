use crate::{PersistenceError, PersistenceResult};
use chrono::{DateTime, Duration, NaiveDate, NaiveDateTime, SecondsFormat, Utc};
use football_domain::SpreadsheetEntityType;
use serde_json::{json, Map, Value};
use std::collections::HashSet;
use uuid::Uuid;

pub(super) fn normalize_spreadsheet_payload(
    entity_type: SpreadsheetEntityType,
    payload: &mut Map<String, Value>,
) -> PersistenceResult<()> {
    match entity_type {
        SpreadsheetEntityType::PlayerTeamPeriod => {
            if text(payload, "valid_from").is_empty() {
                let verified_at = text(payload, "verified_at");
                if !verified_at.is_empty() {
                    let date =
                        parse_spreadsheet_datetime(&verified_at, "verified_at")?.date_naive();
                    payload.insert("valid_from".to_string(), Value::String(date.to_string()));
                    payload.insert(
                        "_derived_valid_from".to_string(),
                        Value::String("verified_at".to_string()),
                    );
                }
            }
        }
        SpreadsheetEntityType::PlayerAbility => {
            for key in ["observed_at", "effective_from", "effective_to"] {
                canonicalize_datetime_field(payload, key)?;
            }
        }
        SpreadsheetEntityType::PlayerAvailability => {
            for key in ["valid_from", "valid_to"] {
                canonicalize_datetime_field(payload, key)?;
            }
            normalize_availability_status(payload);
        }
        SpreadsheetEntityType::PlayerDynamicTag => {
            for key in ["observed_at", "valid_from", "valid_to"] {
                canonicalize_datetime_field(payload, key)?;
            }
            normalize_dynamic_tag_source_type(payload);
        }
        _ => {}
    }
    Ok(())
}

fn canonicalize_datetime_field(
    payload: &mut Map<String, Value>,
    key: &str,
) -> PersistenceResult<()> {
    let current = text(payload, key);
    if current.is_empty() {
        return Ok(());
    }
    let canonical =
        parse_spreadsheet_datetime(&current, key)?.to_rfc3339_opts(SecondsFormat::AutoSi, true);
    if canonical != current {
        payload.insert(key.to_string(), Value::String(canonical));
    }
    Ok(())
}

pub(super) fn normalize_availability_status(payload: &mut Map<String, Value>) {
    let original = text(payload, "availability_status");
    if original.is_empty() {
        return;
    }
    let normalized = match original.trim().to_ascii_lowercase().as_str() {
        "questionable" => "doubtful".to_string(),
        "unavailable" => "unavailable".to_string(),
        "available" | "doubtful" | "injured" | "suspended" | "rested" | "returning" | "unknown" => {
            original.trim().to_ascii_lowercase()
        }
        _ => return,
    };
    if normalized != original {
        payload.insert(
            "_availability_status_original".to_string(),
            Value::String(original),
        );
        payload.insert("availability_status".to_string(), Value::String(normalized));
    }
}

pub(super) fn normalize_dynamic_tag_source_type(payload: &mut Map<String, Value>) {
    let original = text(payload, "source_type");
    if original.is_empty() {
        return;
    }
    let normalized = match original.trim().to_ascii_lowercase().as_str() {
        "manual" | "provider" | "lineup_import" | "ai_analysis" | "match_review"
        | "calculation" => original.trim().to_ascii_lowercase(),
        "official_web_plus_role_model"
        | "public_roster_initialization"
        | "role_model"
        | "model"
        | "computed"
        | "derived" => "calculation".to_string(),
        "official_web" | "web" | "official_source" => "provider".to_string(),
        _ => return,
    };
    if normalized != original {
        payload.insert("_source_type_original".to_string(), Value::String(original));
        payload.insert("source_type".to_string(), Value::String(normalized));
    }
}

pub(super) fn parse_spreadsheet_datetime(
    value: &str,
    key: &str,
) -> PersistenceResult<DateTime<Utc>> {
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

pub(super) fn text(values: &Map<String, Value>, key: &str) -> String {
    values
        .get(key)
        .and_then(Value::as_str)
        .unwrap_or("")
        .trim()
        .to_string()
}

pub(super) fn required_text(values: &Map<String, Value>, key: &str) -> PersistenceResult<String> {
    let v = text(values, key);
    if v.is_empty() {
        Err(PersistenceError::InvalidState(format!(
            "缺少必填字段：{key}"
        )))
    } else {
        Ok(v)
    }
}

pub(super) fn optional_text(values: &Map<String, Value>, key: &str) -> Option<String> {
    let v = text(values, key);
    if v.is_empty() {
        None
    } else {
        Some(v)
    }
}

pub(super) fn default_text(values: &Map<String, Value>, key: &str, default: &str) -> String {
    optional_text(values, key).unwrap_or_else(|| default.to_string())
}

pub(super) fn optional_uuid(
    values: &Map<String, Value>,
    key: &str,
) -> PersistenceResult<Option<Uuid>> {
    let v = text(values, key);
    if v.is_empty() {
        Ok(None)
    } else {
        Uuid::parse_str(&v)
            .map(Some)
            .map_err(|e| PersistenceError::InvalidState(format!("{key} 不是有效 UUID：{e}")))
    }
}

pub(super) fn payload_optional_uuid(
    values: &Map<String, Value>,
    key: &str,
) -> PersistenceResult<Option<Uuid>> {
    match values.get(key) {
        None | Some(Value::Null) => Ok(None),
        Some(Value::String(v)) if v.is_empty() => Ok(None),
        Some(v) => serde_json::from_value(v.clone())
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
    let v = text(values, key);
    if v.is_empty() {
        Ok(None)
    } else {
        NaiveDate::parse_from_str(&v, "%Y-%m-%d")
            .map(Some)
            .map_err(|e| PersistenceError::InvalidState(format!("{key} 日期格式错误：{e}")))
    }
}

pub(super) fn required_date(
    values: &Map<String, Value>,
    key: &str,
) -> PersistenceResult<NaiveDate> {
    optional_date(values, key)?
        .ok_or_else(|| PersistenceError::InvalidState(format!("缺少日期：{key}")))
}

pub(super) fn optional_datetime(
    values: &Map<String, Value>,
    key: &str,
) -> PersistenceResult<Option<DateTime<Utc>>> {
    let value = text(values, key);
    if value.is_empty() {
        Ok(None)
    } else {
        parse_spreadsheet_datetime(&value, key).map(Some)
    }
}

pub(super) fn required_datetime(
    values: &Map<String, Value>,
    key: &str,
) -> PersistenceResult<DateTime<Utc>> {
    optional_datetime(values, key)?
        .ok_or_else(|| PersistenceError::InvalidState(format!("缺少时间：{key}")))
}

pub(super) fn optional_f64(
    values: &Map<String, Value>,
    key: &str,
) -> PersistenceResult<Option<f64>> {
    let v = text(values, key);
    if v.is_empty() {
        Ok(None)
    } else {
        v.parse::<f64>()
            .map(Some)
            .map_err(|e| PersistenceError::InvalidState(format!("{key} 数值格式错误：{e}")))
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
    let v = text(values, key);
    if v.is_empty() {
        Ok(None)
    } else {
        v.parse::<i16>()
            .map(Some)
            .map_err(|e| PersistenceError::InvalidState(format!("{key} 整数格式错误：{e}")))
    }
}

pub(super) fn optional_i32(
    values: &Map<String, Value>,
    key: &str,
) -> PersistenceResult<Option<i32>> {
    let v = text(values, key);
    if v.is_empty() {
        Ok(None)
    } else {
        v.parse::<i32>()
            .map(Some)
            .map_err(|e| PersistenceError::InvalidState(format!("{key} 整数格式错误：{e}")))
    }
}

pub(super) fn optional_bool(
    values: &Map<String, Value>,
    key: &str,
) -> PersistenceResult<Option<bool>> {
    let v = text(values, key).to_lowercase();
    if v.is_empty() {
        Ok(None)
    } else {
        match v.as_str() {
            "true" | "1" | "yes" | "是" => Ok(Some(true)),
            "false" | "0" | "no" | "否" => Ok(Some(false)),
            _ => Err(PersistenceError::InvalidState(format!(
                "{key} 布尔格式错误"
            ))),
        }
    }
}

fn spreadsheet_source_urls(values: &Map<String, Value>) -> Vec<String> {
    optional_text(values, "source_urls")
        .map(|value| {
            value
                .split(['\n', ';'])
                .map(str::trim)
                .filter(|item| !item.is_empty())
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default()
}

pub(super) fn spreadsheet_clear_fields(values: &Map<String, Value>) -> HashSet<String> {
    optional_text(values, "clear_fields")
        .map(|value| {
            value
                .split([',', '，', ';'])
                .map(str::trim)
                .filter(|item| !item.is_empty())
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default()
}

pub(super) fn spreadsheet_row_metadata(values: &Map<String, Value>) -> Value {
    let is_monthly_workbook =
        values.contains_key("source_urls") || values.contains_key("verified_at");
    json!({
        "source": "spreadsheet",
        "monthly_workbook": is_monthly_workbook,
        "source_urls": spreadsheet_source_urls(values),
        "verified_at": optional_text(values, "verified_at"),
        "confidence": optional_text(values, "confidence")
            .and_then(|value| value.parse::<f64>().ok()),
        "notes": optional_text(values, "notes"),
    })
}

pub(super) fn normalize_name(value: &str) -> String {
    value
        .trim()
        .to_lowercase()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn spreadsheet_datetime_accepts_date_only_and_excel_serial() {
        let date_only =
            parse_spreadsheet_datetime("2026-07-18", "observed_at").expect("date-only timestamp");
        assert_eq!(
            date_only.to_rfc3339_opts(SecondsFormat::Secs, true),
            "2026-07-18T00:00:00Z"
        );
        let serial =
            parse_spreadsheet_datetime("46221", "observed_at").expect("Excel serial timestamp");
        assert_eq!(serial.date_naive().to_string(), "2026-07-18");
    }
    #[test]
    fn player_team_period_derives_start_date_from_verification_time() {
        let mut payload = json!({
            "valid_from": "",
            "verified_at": "2026-07-18T15:30:00Z"
        })
        .as_object()
        .expect("object")
        .clone();
        normalize_spreadsheet_payload(SpreadsheetEntityType::PlayerTeamPeriod, &mut payload)
            .expect("normalize period");
        assert_eq!(text(&payload, "valid_from"), "2026-07-18");
        assert_eq!(text(&payload, "_derived_valid_from"), "verified_at");
    }
    #[test]
    fn dynamic_tag_source_alias_is_normalized() {
        let mut payload = json!({"source_type": "official_web_plus_role_model"})
            .as_object()
            .expect("object")
            .clone();
        normalize_dynamic_tag_source_type(&mut payload);
        assert_eq!(text(&payload, "source_type"), "calculation");
        assert_eq!(
            text(&payload, "_source_type_original"),
            "official_web_plus_role_model"
        );
    }
    #[test]
    fn public_roster_initialization_source_is_audited_as_calculation() {
        let mut payload = json!({"source_type": "public_roster_initialization"})
            .as_object()
            .expect("object")
            .clone();
        normalize_dynamic_tag_source_type(&mut payload);
        assert_eq!(text(&payload, "source_type"), "calculation");
        assert_eq!(
            text(&payload, "_source_type_original"),
            "public_roster_initialization"
        );
    }
    #[test]
    fn availability_aliases_are_normalized_without_losing_original_value() {
        let mut questionable = json!({"availability_status": "questionable"})
            .as_object()
            .expect("object")
            .clone();
        normalize_availability_status(&mut questionable);
        assert_eq!(text(&questionable, "availability_status"), "doubtful");
        assert_eq!(
            text(&questionable, "_availability_status_original"),
            "questionable"
        );

        let mut unavailable = json!({"availability_status": "unavailable"})
            .as_object()
            .expect("object")
            .clone();
        normalize_availability_status(&mut unavailable);
        assert_eq!(text(&unavailable, "availability_status"), "unavailable");
    }

    #[test]
    fn payload_dates_uuid_and_clear_fields_keep_distinct_semantics() {
        let payload = json!({"player_id":"bad-id", "valid_from":"bad-date", "confidence":"bad-number", "source_urls":" https://a.test ;\n https://b.test ", "clear_fields":"birth_date，height_cm; preferred_foot"}).as_object().unwrap().clone();
        assert!(optional_uuid(&payload, "player_id")
            .unwrap_err()
            .to_string()
            .contains("player_id 不是有效 UUID"));
        assert!(optional_date(&payload, "valid_from").is_err());
        assert!(optional_f64(&payload, "confidence").is_err());
        assert_eq!(
            spreadsheet_source_urls(&payload),
            vec!["https://a.test", "https://b.test"]
        );
        assert_eq!(
            spreadsheet_clear_fields(&payload),
            HashSet::from([
                "birth_date".into(),
                "height_cm".into(),
                "preferred_foot".into()
            ])
        );
        let empty = Map::new();
        assert_eq!(optional_uuid(&empty, "player_id").unwrap(), None);
        assert!(payload_uuid(&empty, "_resolved_player_id").is_err());
        for bad in ["NaN", "0", "1000001", "2026-99-99"] {
            assert!(parse_spreadsheet_datetime(bad, "observed_at").is_err());
        }
    }
}
