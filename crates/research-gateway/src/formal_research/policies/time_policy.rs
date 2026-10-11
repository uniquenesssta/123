use crate::validation_error::schema_error;
use crate::GatewayError;
use chrono::{DateTime, Utc};

pub(crate) fn validate_time(
    field: &str,
    value: Option<DateTime<Utc>>,
    cutoff: DateTime<Utc>,
) -> Result<(), GatewayError> {
    if value.is_some_and(|timestamp| timestamp > cutoff) {
        return Err(schema_error(format!(
            "{field}晚于赛前数据截止时间，不能进入当前研究结果"
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::GatewayErrorCategory;
    use chrono::{Duration, TimeZone};

    #[test]
    fn optional_fact_times_accept_none_before_and_equal_but_reject_one_nanosecond_after() {
        let cutoff = Utc.with_ymd_and_hms(2026, 7, 14, 10, 0, 0).unwrap();
        for field in ["published_at", "observed_at", "effective_at"] {
            validate_time(field, None, cutoff).expect("optional");
            for offset in [-1, 0] {
                validate_time(field, Some(cutoff + Duration::nanoseconds(offset)), cutoff)
                    .expect("at or before cutoff");
            }
            let error = validate_time(field, Some(cutoff + Duration::nanoseconds(1)), cutoff)
                .expect_err("future");
            assert_eq!(error.category, GatewayErrorCategory::SchemaValidation);
            assert_eq!(
                error.user_message,
                format!("{field}晚于赛前数据截止时间，不能进入当前研究结果")
            );
            assert!(!error.recovery.retryable);
            assert_eq!(
                error.recovery.action,
                "保留原始响应并将任务标记为结构校验失败；修正Prompt或Schema版本后重试"
            );
        }
    }

    #[test]
    fn cutoff_compares_utc_instants_without_wall_clock_or_string_timezone_rules() {
        let cutoff = DateTime::parse_from_rfc3339("2026-07-14T10:00:00.123456789Z")
            .unwrap()
            .with_timezone(&Utc);
        for time in [
            "2026-07-14T18:00:00.123456789+08:00",
            "2026-07-14T03:00:00.123456788-07:00",
        ] {
            validate_time(
                "observed_at",
                Some(
                    DateTime::parse_from_rfc3339(time)
                        .unwrap()
                        .with_timezone(&Utc),
                ),
                cutoff,
            )
            .expect("same or earlier instant");
        }
        let future = DateTime::parse_from_rfc3339("2026-07-14T18:00:00.123456790+08:00")
            .unwrap()
            .with_timezone(&Utc);
        assert!(validate_time("observed_at", Some(future), cutoff).is_err());
    }
}
