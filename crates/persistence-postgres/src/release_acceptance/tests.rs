use super::mapping::{parse_status, report_count};
use crate::PersistenceError;
use football_domain::ReleaseAcceptanceStatus;

#[test]
fn stored_status_accepts_exact_three_original_values() {
    for (value, expected) in [
        ("pass", ReleaseAcceptanceStatus::Pass),
        ("warning", ReleaseAcceptanceStatus::Warning),
        ("blocked", ReleaseAcceptanceStatus::Blocked),
    ] {
        assert_eq!(parse_status(value).unwrap(), expected);
    }
}

#[test]
fn unknown_status_preserves_original_untrimmed_error() {
    for value in ["", " pass", "warning ", "Pass", "unknown", "未知"] {
        assert!(
            matches!(parse_status(value).unwrap_err(),PersistenceError::InvalidState(message)
            if message==format!("未知发布验收状态：{value}"))
        );
    }
}

#[test]
fn legacy_negative_report_counts_remain_zero_without_new_rejection() {
    for value in [i32::MIN, -1, 0] {
        assert_eq!(report_count(value), 0);
    }
}

#[test]
fn positive_report_counts_preserve_original_full_i32_range() {
    for value in [1, 17, i32::MAX] {
        assert_eq!(report_count(value), value as u32);
    }
}
